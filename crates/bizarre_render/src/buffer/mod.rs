use std::{
    ops::{Deref, DerefMut, RangeBounds},
    ptr::slice_from_raw_parts_mut,
};

use ash::vk;
use thiserror::Error;
use vma::Alloc;

use crate::device::VulkanDevice;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error(transparent)]
    VkError(#[from] vk::Result),
    #[error("Could not transfer data between buffers: {0}")]
    TransferError(#[from] BufferTransferError),
}

#[derive(Debug, Error)]
pub enum BufferTransferError {
    #[error(
        "{}",
        format_transfer_flags(*no_transfer_src, *no_transfer_dst)
    )]
    NoTransferFlags {
        no_transfer_src: bool,
        no_transfer_dst: bool,
    },
    #[error("Transfer destination is too small, copy size: {src_size}, destination offset: {dst_offset}, available size after destination offset: {}", dst_size - dst_offset)]
    TransferDstTooSmall {
        src_size: usize,
        dst_offset: usize,
        dst_size: usize,
    },
}

pub type BufferResult<T> = Result<T, BufferError>;

pub struct GpuBuffer {
    buffer: vk::Buffer,
    allocation: vma::Allocation,
    size: usize,
    buffer_usage: vk::BufferUsageFlags,
    mem_usage: vma::MemoryUsage,
    alloc_flags: vma::AllocationCreateFlags,
}

impl GpuBuffer {
    pub fn staging_buffer(device: &VulkanDevice, size: usize) -> BufferResult<Self> {
        Self::new(
            device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vma::MemoryUsage::AutoPreferHost,
            vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        )
    }

    pub fn new(
        device: &VulkanDevice,
        size: usize,
        buffer_usage: vk::BufferUsageFlags,
        mem_usage: vma::MemoryUsage,
        alloc_flags: vma::AllocationCreateFlags,
    ) -> BufferResult<Self> {
        let (buffer, allocation) = {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(size as vk::DeviceSize)
                .usage(buffer_usage);
            let allocation_info = vma::AllocationCreateInfo {
                usage: mem_usage,
                flags: alloc_flags,
                ..Default::default()
            };
            unsafe {
                device
                    .allocator
                    .create_buffer(&buffer_info, &allocation_info)?
            }
        };

        Ok(Self {
            buffer,
            allocation,
            size,
            alloc_flags,
            buffer_usage,
            mem_usage,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn buffer(&self) -> &vk::Buffer {
        &self.buffer
    }

    pub fn allocation(&self) -> &vma::Allocation {
        &self.allocation
    }

    pub fn buffer_usage(&self) -> &vk::BufferUsageFlags {
        &self.buffer_usage
    }

    pub fn memory_usage(&self) -> &vma::MemoryUsage {
        &self.mem_usage
    }

    pub fn allocation_flags(&self) -> &vma::AllocationCreateFlags {
        &self.alloc_flags
    }

    pub fn copy_from_buffer_range<R: RangeBounds<u64>>(
        &mut self,
        device: &VulkanDevice,
        src: &GpuBuffer,
        src_range: R,
        dst_offset: u64,
    ) -> BufferResult<()> {
        let no_transfer_src = src
            .buffer_usage
            .intersects(vk::BufferUsageFlags::TRANSFER_SRC);
        let no_transfer_dst = self
            .buffer_usage
            .intersects(vk::BufferUsageFlags::TRANSFER_DST);

        if no_transfer_dst || no_transfer_src {
            return Err(BufferTransferError::NoTransferFlags {
                no_transfer_src,
                no_transfer_dst,
            }
            .into());
        }

        let src_offset = match src_range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Unbounded => 0,
            std::ops::Bound::Excluded(_) => unreachable!(),
        };

        let src_size = match src_range.end_bound() {
            std::ops::Bound::Included(end) => end - src_offset,
            std::ops::Bound::Excluded(end) => end - 1 - src_offset,
            std::ops::Bound::Unbounded => src.size as u64,
        };

        unsafe {
            self.copy_from_buffer_raw(device, src.buffer, &[(src_offset, src_size)], &[dst_offset])
        }
    }

    pub fn copy_from_buffer_ranges<R: RangeBounds<u64>>(
        &mut self,
        device: &VulkanDevice,
        src: &GpuBuffer,
        src_ranges: &[R],
        dst_offsets: &[u64],
    ) -> BufferResult<()> {
        let no_transfer_src = src
            .buffer_usage
            .intersects(vk::BufferUsageFlags::TRANSFER_SRC);
        let no_transfer_dst = self
            .buffer_usage
            .intersects(vk::BufferUsageFlags::TRANSFER_DST);

        if no_transfer_dst || no_transfer_src {
            return Err(BufferTransferError::NoTransferFlags {
                no_transfer_src,
                no_transfer_dst,
            }
            .into());
        }

        let src_ranges = src_ranges.iter().map(|range| {
            let offset = match range.start_bound() {
                std::ops::Bound::Included(start) => *start,
                std::ops::Bound::Unbounded => 0,
                std::ops::Bound::Excluded(_) => unreachable!(),
            };
            let size = match range.end_bound() {
                std::ops::Bound::Included(end) => end - offset,
                std::ops::Bound::Excluded(end) => end - 1 - offset,
                std::ops::Bound::Unbounded => src.size as u64 - offset,
            };

            (offset, size)
        });

        unsafe {
            return self.copy_from_buffer_raw(
                device,
                src.buffer,
                &src_ranges.collect::<Vec<_>>(),
                dst_offsets,
            );
        }
    }

    pub fn copy_from_buffer(&mut self, device: &VulkanDevice, src: &Self) -> BufferResult<()> {
        if self.size < src.size {
            self.grow(device, src.size)?;
        }

        let no_transfer_src = src
            .buffer_usage
            .intersects(vk::BufferUsageFlags::TRANSFER_SRC);
        let no_transfer_dst = self
            .buffer_usage
            .intersects(vk::BufferUsageFlags::TRANSFER_DST);

        if no_transfer_dst || no_transfer_src {
            return Err(BufferTransferError::NoTransferFlags {
                no_transfer_src,
                no_transfer_dst,
            }
            .into());
        }

        if src.size > self.size {
            return Err(BufferTransferError::TransferDstTooSmall {
                src_size: src.size,
                dst_offset: 0,
                dst_size: self.size,
            }
            .into());
        }

        unsafe { self.copy_from_buffer_raw(device, src.buffer, &[(0, src.size as u64)], &[0]) }
    }

    /// Copy from another buffer
    ///
    /// * `device` - VulkanDevice
    /// * `src` - Other buffer to copy from
    /// * `src_ranges` - Source ranges to copy from in form of `(src_offset, src_size)` corresponding to the dst_offsets
    /// * `dst_offsets` - Offset inside a destination (self) corresponding to the source ranges
    pub unsafe fn copy_from_buffer_raw(
        &mut self,
        device: &VulkanDevice,
        src: vk::Buffer,
        src_ranges: &[(u64, u64)],
        dst_offsets: &[u64],
    ) -> BufferResult<()> {
        let buffer_create_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(device.cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let cmd_buffer = device.allocate_command_buffers(&buffer_create_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo::default();

        device.begin_command_buffer(cmd_buffer, &begin_info)?;

        let regions = src_ranges
            .iter()
            .zip(dst_offsets.iter())
            .map(|((src_offset, src_size), dst_offset)| {
                vk::BufferCopy2::default()
                    .src_offset(*src_offset)
                    .size(*src_size)
                    .dst_offset(*dst_offset)
            })
            .collect::<Vec<_>>();

        let copy_info = vk::CopyBufferInfo2::default()
            .src_buffer(src)
            .dst_buffer(self.buffer)
            .regions(&regions);

        device.cmd_copy_buffer2(cmd_buffer, &copy_info);

        device.end_command_buffer(cmd_buffer)?;

        let cmd_buffers = [cmd_buffer];

        let submit = vk::SubmitInfo::default().command_buffers(&cmd_buffers);

        let submits = [submit];

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let fence = device.create_fence(&fence_create_info, None)?;

        device.queue_submit(device.compute_queue, &submits, fence)?;

        device.wait_for_fences(&[fence], true, u64::MAX)?;

        Ok(())
    }

    pub fn grow(&mut self, device: &VulkanDevice, size: usize) -> BufferResult<()> {
        if size < self.size {
            return Ok(());
        }

        let mut new_buffer = Self::new(
            device,
            size,
            self.buffer_usage,
            self.mem_usage,
            self.alloc_flags,
        )?;

        new_buffer.copy_from_buffer(device, &self)?;

        self.destroy(device);

        *self = new_buffer;

        Ok(())
    }

    pub fn destroy(&mut self, device: &VulkanDevice) {
        unsafe {
            device
                .allocator
                .destroy_buffer(self.buffer, &mut self.allocation);
        }
    }

    pub fn map_as_slice<'a, T>(
        &'a mut self,
        device: &'a VulkanDevice,
    ) -> BufferResult<MappedSlice<'a, T>> {
        let slice = unsafe {
            let ptr = device.allocator.map_memory(&mut self.allocation)? as *mut T;
            let len = self.size / size_of::<T>();
            &mut *slice_from_raw_parts_mut(ptr, len)
        };

        let mapped_allocation = MappedSlice {
            allocator: &device.allocator,
            allocation: &mut self.allocation,
            data: slice,
        };

        Ok(mapped_allocation)
    }
}

pub struct MappedSlice<'a, T> {
    allocator: &'a vma::Allocator,
    allocation: &'a mut vma::Allocation,
    data: &'a mut [T],
}

impl<T> Drop for MappedSlice<'_, T> {
    fn drop(&mut self) {
        unsafe { self.allocator.unmap_memory(self.allocation) }
    }
}

impl<'a, T> Deref for MappedSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> DerefMut for MappedSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

fn format_transfer_flags(no_src_transfer: bool, no_dst_transfer: bool) -> String {
    let src_label = "no TRANSFER_SRC on source buffer";
    let dst_label = "no TRANSFER_DST on destination buffer";

    [(no_src_transfer, src_label), (no_dst_transfer, dst_label)]
        .into_iter()
        .filter_map(|(flag, label)| if flag { Some(label) } else { None })
        .collect::<Vec<_>>()
        .join(", ")
}
