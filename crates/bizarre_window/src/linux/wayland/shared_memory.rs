use std::{
    ops::{Bound, RangeBounds},
    os::fd::{AsFd, OwnedFd},
    ptr,
};

use rustix::{
    fs::ftruncate,
    mm::{munmap, MapFlags, ProtFlags},
    shm,
};

pub struct SharedMemory {
    name: String,
    size: usize,
    fd: OwnedFd,
    o_flags: shm::OFlags,
    mode: shm::Mode,
}

impl SharedMemory {
    pub fn new(
        name: String,
        size: usize,
        flags: shm::OFlags,
        mode: shm::Mode,
    ) -> rustix::io::Result<Self> {
        let fd = shm::open(&name, flags, mode)?;

        ftruncate(fd.as_fd(), size as u64)?;

        Ok(Self {
            name,
            size,
            fd,
            o_flags: flags,
            mode,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn resize(&mut self, new_size: usize) {
        if new_size <= self.size {
            return;
        }
        ftruncate(self.as_fd(), new_size as u64);
    }

    pub unsafe fn map<T, R: RangeBounds<usize>>(
        &self,
        prot_flags: ProtFlags,
        map_flags: MapFlags,
        range: R,
    ) -> *mut T {
        let start = match range.start_bound() {
            Bound::Included(value) => *value,
            Bound::Excluded(value) => value + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(val) => val + 1,
            Bound::Excluded(val) => *val,
            Bound::Unbounded => self.size,
        };

        let len = end - start;

        rustix::mm::mmap(
            ptr::null_mut(),
            len,
            prot_flags,
            map_flags,
            self.as_fd(),
            start as u64,
        )
        .unwrap()
        .cast::<T>()
    }

    pub unsafe fn unmap<T>(&self, ptr: *mut T, size: usize) {
        munmap(ptr.cast(), size).unwrap()
    }
}

impl AsFd for SharedMemory {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        shm::unlink(&self.name);
    }
}

#[cfg(test)]
mod test {
    use rustix::shm::OFlags;

    use super::SharedMemory;

    #[test]
    fn should_resize_shm_to_bigger() {
        let shm = SharedMemory::new(
            "/test_shm".into(),
            256,
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR,
            rustix::fs::Mode::from_bits_retain(600),
        )
        .unwrap();
    }
}
