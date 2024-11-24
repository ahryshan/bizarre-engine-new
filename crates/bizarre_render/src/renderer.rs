use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;

use ash::vk;
use ash::vk::Handle as _;
use bizarre_ecs::prelude::Resource;
use bizarre_log::core_info;
use bizarre_window::Window;
use nalgebra_glm::UVec2;
use thiserror::Error;

use crate::material::pipeline::PipelineError;
use crate::material::pipeline::PipelineHandle;
use crate::material::pipeline::VulkanPipeline;
use crate::material::pipeline::VulkanPipelineRequirements;
use crate::present_target::PresentData;
use crate::present_target::PresentError;
use crate::present_target::PresentResult;
use crate::render_pass::basic_render_pass;
use crate::render_pass::RenderPass;
use crate::render_pass::RenderPassHandle;
use crate::render_target::ImageRenderTarget;
use crate::render_target::RenderData;
use crate::render_target::RenderTarget;
use crate::render_target::RenderTargetHandle;
use crate::render_target::SwapchainRenderTarget;
use crate::submitter::RenderPackage;
use crate::{
    device::{DeviceError, VulkanDevice},
    instance::{InstanceError, VulkanInstance},
    present_target::{PresentTarget, PresentTargetData},
};

#[derive(Debug, Default)]
pub(crate) struct RenderPasses {
    basic: Option<vk::RenderPass>,
    custom: HashMap<RenderPassHandle, vk::RenderPass>,
}

#[derive(Resource)]
pub struct VulkanRenderer {
    device: VulkanDevice,
    instance: VulkanInstance,
    present_targets: HashMap<PresentTarget, PresentTargetData>,
    render_targets: HashMap<RenderTargetHandle, Box<dyn RenderTarget>>,
    next_render_target_id: AtomicUsize,
    pipelines: HashMap<PipelineHandle, VulkanPipeline>,
    render_passes: RenderPasses,
    render_cmd_buffer: vk::CommandBuffer,
    present_cmd_buffer: vk::CommandBuffer,
    swapchain_loader: ash::khr::swapchain::Device,
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error(transparent)]
    VulkanError(#[from] vk::Result),
    #[error("Failed to create a `VulkanRenderer`: {0}")]
    CreateError(#[from] RendererCreateError),
    #[error(transparent)]
    PipelineError(#[from] PipelineError),
}
#[derive(Error, Debug)]
pub enum RendererCreateError {
    #[error(transparent)]
    InstanceError(#[from] InstanceError),
    #[error(transparent)]
    DeviceError(#[from] DeviceError),
}

pub type RenderResult<T> = Result<T, RenderError>;

impl VulkanRenderer {
    pub fn new() -> RenderResult<Self> {
        let instance = VulkanInstance::new();
        let device = instance
            .create_device_ext()
            .map_err(|err| RenderError::CreateError(err.into()))?;

        core_info!("Created a renderer");

        let basic_render_pass = basic_render_pass(&device)?;

        let cmd_buffers = {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_buffer_count(2)
                .command_pool(device.cmd_pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            unsafe { device.allocate_command_buffers(&allocate_info)? }
        };

        let (render_cmd_buffer, present_cmd_buffer) =
            if let &[render_cmd, present_cmd, ..] = cmd_buffers.as_slice() {
                (render_cmd, present_cmd)
            } else {
                unreachable!();
            };

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        Ok(Self {
            instance,
            device,
            present_targets: Default::default(),
            render_targets: Default::default(),
            next_render_target_id: AtomicUsize::new(1),
            pipelines: Default::default(),
            render_passes: RenderPasses {
                basic: Some(basic_render_pass),
                custom: Default::default(),
            },
            render_cmd_buffer,
            present_cmd_buffer,
            swapchain_loader,
        })
    }

    pub fn add_window(
        &mut self,
        window: &Window,
        render_pass: RenderPass,
    ) -> RenderResult<(RenderTargetHandle, PresentTarget)> {
        let present_target = self.create_present_target(window, render_pass)?;
        let image_count = self
            .present_targets
            .get(&present_target)
            .unwrap()
            .image_count();

        let render_target =
            self.create_swapchain_render_target(window.size(), render_pass, image_count)?;

        Ok((render_target, present_target))
    }

    pub fn create_swapchain_render_target(
        &mut self,
        extent: UVec2,
        render_pass: RenderPass,
        image_count: u32,
    ) -> RenderResult<RenderTargetHandle> {
        let vk_render_pass = self.get_render_pass(render_pass).unwrap();
        let render_target = SwapchainRenderTarget::new(
            &self.device,
            extent,
            self.device.cmd_pool,
            vk_render_pass,
            render_pass,
            image_count,
        )?;

        let handle = RenderTargetHandle::from_raw(
            self.next_render_target_id
                .fetch_add(1, atomic::Ordering::SeqCst),
        );

        self.render_targets.insert(handle, Box::new(render_target));

        Ok(handle)
    }

    pub fn create_present_target(
        &mut self,
        window: &Window,
        render_pass: RenderPass,
    ) -> RenderResult<PresentTarget> {
        let target = PresentTarget::from_raw(window.handle().as_raw());
        let data = unsafe {
            let display = bizarre_window::get_wayland_display_ptr() as *mut vk::wl_display;
            let surface = window.raw_window_ptr() as *mut c_void;
            let render_pass = self.get_render_pass(render_pass).expect(&format!(
                "Failed to find `{:?}` renderpass in this renderer",
                render_pass,
            ));

            PresentTargetData::new(
                &self.instance,
                &self.device,
                self.device.cmd_pool,
                render_pass,
                window.size(),
                display,
                surface,
            )?
        };

        self.present_targets.insert(target, data);

        Ok(target)
    }

    pub fn resize_present_target(
        &mut self,
        present_target: PresentTarget,
        size: UVec2,
    ) -> PresentResult<()> {
        let present_target = self
            .present_targets
            .get_mut(&present_target)
            .ok_or(PresentError::InvalidPresentTarget)?;

        present_target.resize(&self.device, size)
    }

    pub fn create_render_target(&mut self) -> RenderResult<RenderTargetHandle> {
        todo!()
    }

    pub fn render(
        &mut self,
        render_target: RenderTargetHandle,
        render_package: RenderPackage,
    ) -> RenderResult<()> {
        let render_target = self.render_targets.get(&render_target).unwrap();

        let RenderData {
            in_flight_fence,
            render_ready,
            cmd_buffer,
            framebuffer,
            extent,
            ..
        } = render_target.get_render_data();

        let fences = [in_flight_fence];
        let cmd_buffer = cmd_buffer;
        let render_pass = self.render_passes.basic.unwrap();

        unsafe {
            self.device.wait_for_fences(&fences, true, u64::MAX)?;

            let begin_info = vk::CommandBufferBeginInfo::default();
            self.device.begin_command_buffer(cmd_buffer, &begin_info)?;

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.2, 0.25, 1.0],
                },
            }];

            let render_pass_info = vk::RenderPassBeginInfo::default()
                .clear_values(&clear_values)
                .render_pass(render_pass)
                .framebuffer(framebuffer)
                .render_area(vk::Rect2D {
                    extent,
                    offset: vk::Offset2D { x: 0, y: 0 },
                });

            self.device.cmd_begin_render_pass(
                cmd_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            let viewport = vk::Viewport::default()
                .x(0.0)
                .y(extent.height as f32)
                .width(extent.width as f32)
                .height(-(extent.height as f32))
                .min_depth(0.0)
                .max_depth(1.0);

            self.device.cmd_set_viewport(cmd_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            self.device.cmd_set_scissor(cmd_buffer, 0, &[scissor]);

            let pipeline = self.pipelines.get(&render_package.pipeline).unwrap();

            self.device.cmd_bind_pipeline(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.handle,
            );

            self.device.cmd_draw(cmd_buffer, 3, 1, 0, 0);

            self.device.cmd_end_render_pass(cmd_buffer);
            self.device.end_command_buffer(cmd_buffer)?;

            let submit_buffers = [cmd_buffer];
            let submit_signal = [render_ready];

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&submit_buffers)
                .signal_semaphores(&submit_signal);

            let submits = [submit_info];

            self.device.reset_fences(&[in_flight_fence])?;

            self.device
                .queue_submit(self.device.graphics_queue, &submits, in_flight_fence)?;
        }

        Ok(())
    }

    pub fn present(
        &mut self,
        present_target: PresentTarget,
        render_target: RenderTargetHandle,
    ) -> PresentResult<()> {
        unsafe { self.device.device_wait_idle() }?;

        let present_target = self.present_targets.get_mut(&present_target).unwrap();
        let render_target = self.render_targets.get_mut(&render_target).unwrap();

        let PresentData {
            cmd_buffer,
            swapchain,
            image_acquired,
            image_ready,
            image_index: index,
        } = present_target
            .record_present(&self.device, render_target.image())
            .unwrap();

        let swapchains = [swapchain];
        let indices = [index];
        let buffers = [cmd_buffer].into_iter().flatten().collect::<Vec<_>>();

        let cmd_wait = [image_acquired, render_target.render_complete_semaphore()];
        let images_ready = [image_ready];

        let pipeline_stage_masks = [
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
        ];

        unsafe {
            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&buffers)
                .signal_semaphores(&images_ready)
                .wait_semaphores(&cmd_wait)
                .wait_dst_stage_mask(&pipeline_stage_masks);

            let submits = [submit_info];

            self.device
                .queue_submit(self.device.present_queue, &submits, vk::Fence::null())?;
        };

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .image_indices(&indices)
            .wait_semaphores(&images_ready);

        unsafe {
            self.swapchain_loader
                .queue_present(self.device.present_queue, &present_info)?
        };

        render_target.next_frame();

        Ok(())
    }

    pub fn new_pipeline(
        &mut self,
        requirements: &VulkanPipelineRequirements,
    ) -> RenderResult<PipelineHandle> {
        let render_pass = self
            .get_render_pass(requirements.render_pass)
            .expect(&format!(
                "Failed to find `{:?}` renderpass in this renderer",
                requirements.render_pass
            ));

        let pipeline =
            VulkanPipeline::from_requirements(requirements, None, render_pass, &self.device)?;
        let handle = PipelineHandle::from_raw(pipeline.handle.as_raw());
        self.pipelines.insert(handle, pipeline);
        Ok(handle)
    }
    pub fn new_custom_render_pass<F>(&mut self, constructor: F) -> RenderResult<RenderPassHandle>
    where
        F: Fn(&VulkanDevice) -> Result<vk::RenderPass, vk::Result>,
    {
        let render_pass = constructor(&self.device)?;
        let handle = RenderPassHandle::from_raw(render_pass.as_raw());
        self.render_passes.custom.insert(handle, render_pass);
        Ok(handle)
    }

    fn get_render_pass(&self, render_pass: RenderPass) -> Option<vk::RenderPass> {
        match render_pass {
            RenderPass::Basic => self.render_passes.basic.clone(),
            RenderPass::Custom(handle) => self.render_passes.custom.get(&handle).cloned(),
        }
    }

    fn destroy_render_passes(&mut self) {
        if let Some(render_pass) = self.render_passes.basic.take() {
            unsafe { self.device.destroy_render_pass(render_pass, None) };
        }

        self.render_passes
            .custom
            .drain()
            .for_each(|(_, render_pass)| unsafe {
                self.device.destroy_render_pass(render_pass, None)
            });
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
        }

        self.pipelines
            .drain()
            .for_each(|(_, mut pipeline)| pipeline.destroy(&self.device));

        self.destroy_render_passes();

        self.render_targets
            .drain()
            .for_each(|(_, mut render_target)| render_target.destroy(&self.device));

        self.present_targets
            .drain()
            .for_each(|(_, mut target)| target.destroy(&self.device));
    }
}
