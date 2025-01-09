use core::fmt::Debug;
use std::ffi::c_void;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::{collections::HashMap, path::Path};

use ash::vk;
use bizarre_core::handle::DenseHandleStrategy;
use bizarre_ecs::prelude::Resource;
use bizarre_window::Window;
use nalgebra_glm::UVec2;
use thiserror::Error;

use crate::material::material_instance::{MaterialInstance, MaterialInstanceHandle};
use crate::material::pipeline::{VulkanPipeline, VulkanPipelineRequirements};
use crate::material::{Material, MaterialHandle};
use crate::{
    antialiasing::Antialiasing,
    asset_manager::AssetStore,
    device::logical_device::DeviceError,
    instance::InstanceError,
    material::pipeline::PipelineError,
    mesh::{Mesh, MeshHandle},
    present_target::{
        PresentData, PresentError, PresentResult, PresentTarget, PresentTargetHandle,
    },
    render_target::{RenderTargetHandle, SwapchainRenderTarget},
    scene::{object_pass::SceneObjectPass, Scene, SceneHandle},
    submitter::RenderPackage,
    vulkan_context::{get_device, get_instance},
};

#[derive(Resource)]
pub struct VulkanRenderer {
    max_frames_in_flight: u32,
    swapchain_loader: ash::khr::swapchain::Device,
    antialiasing: Antialiasing,

    present_targets: HashMap<PresentTargetHandle, PresentTarget>,
    render_targets: AssetStore<SwapchainRenderTarget, DenseHandleStrategy<SwapchainRenderTarget>>,
    scenes: AssetStore<Scene, DenseHandleStrategy<Scene>>,
    meshes: AssetStore<Mesh, DenseHandleStrategy<Mesh>>,
    materials: AssetStore<Material, DenseHandleStrategy<Material>>,
    material_instances: AssetStore<MaterialInstance, DenseHandleStrategy<MaterialInstance>>,
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error(transparent)]
    VulkanError(#[from] vk::Result),
    #[error("Failed to create a `VulkanRenderer`: {0}")]
    CreateError(#[from] RendererCreateError),
    #[error(transparent)]
    PipelineError(#[from] PipelineError),
    #[error("Invalid render target")]
    InvalidRenderTarget,
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
        let instance = get_instance();
        let device = get_device();

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        Ok(Self {
            // TODO: Make it dynamic and/or configurable
            max_frames_in_flight: 3,
            present_targets: Default::default(),
            render_targets: Default::default(),
            swapchain_loader,
            antialiasing: Antialiasing::None,
            scenes: Default::default(),
            meshes: Default::default(),
            materials: Default::default(),
            material_instances: Default::default(),
        })
    }

    pub fn create_swapchain_render_target(
        &mut self,
        extent: UVec2,
        image_count: u32,
    ) -> RenderResult<RenderTargetHandle> {
        let device = get_device();

        let render_target = SwapchainRenderTarget::new(
            device,
            extent,
            device.cmd_pool,
            self.antialiasing.into(),
            image_count,
        )?;

        let handle = self.render_targets.insert(render_target);

        Ok(handle)
    }

    pub fn create_present_target(&mut self, window: &Window) -> RenderResult<PresentTargetHandle> {
        let device = get_device();
        let instance = get_instance();

        let target = PresentTargetHandle::from_raw(window.handle().as_raw());
        let data = unsafe {
            let display = bizarre_window::get_wayland_display_ptr() as *mut vk::wl_display;
            let surface = window.raw_window_ptr() as *mut c_void;

            PresentTarget::new(
                instance,
                device,
                device.cmd_pool,
                window.size(),
                display,
                surface,
            )?
        };

        self.present_targets.insert(target, data);

        Ok(target)
    }

    pub fn load_mesh<P>(&mut self, path: P) -> RenderResult<MeshHandle>
    where
        P: AsRef<Path> + Debug,
    {
        let mesh = Mesh::load_from_obj(path);

        let handle = self.meshes.insert(mesh);

        Ok(handle)
    }

    pub fn create_material(
        &mut self,
        pipeline_requirements: VulkanPipelineRequirements,
    ) -> RenderResult<MaterialHandle> {
        let pipeline =
            VulkanPipeline::from_requirements(&pipeline_requirements, None, get_device())?;

        let material = Material::new(pipeline, pipeline_requirements.bindings);

        let handle = self.materials.insert(material);

        Ok(handle)
    }

    pub fn insert_material(&mut self, material: Material) -> MaterialHandle {
        self.materials.insert(material)
    }

    pub fn create_material_instance(
        &mut self,
        material: MaterialHandle,
    ) -> RenderResult<MaterialInstanceHandle> {
        let material_handle = material;
        let material = self.materials.get(&material).unwrap();

        let instance = MaterialInstance::new(material_handle, material).unwrap();

        let handle = self.material_instances.insert(instance);

        Ok(handle)
    }

    pub fn present_target(&self, handle: &PresentTargetHandle) -> Option<&PresentTarget> {
        self.present_targets.get(handle)
    }

    pub fn resize_present_target(
        &mut self,
        present_target: PresentTargetHandle,
        size: UVec2,
    ) -> PresentResult<()> {
        let present_target = self
            .present_targets
            .get_mut(&present_target)
            .ok_or(PresentError::InvalidPresentTarget)?;

        present_target.resize(get_device(), size)
    }

    pub fn create_render_target(&mut self) -> RenderResult<RenderTargetHandle> {
        todo!()
    }

    pub fn create_scene(&mut self, max_frames_in_flight: usize) -> RenderResult<SceneHandle> {
        let handle = self
            .scenes
            .insert(Scene::new(max_frames_in_flight).unwrap());

        Ok(handle)
    }

    pub fn with_scene_mut<F>(&mut self, handle: &SceneHandle, func: F)
    where
        F: FnOnce(&mut Scene),
    {
        let scene = self.scenes.get_mut(handle).unwrap();

        func(scene)
    }

    pub fn render_to_target(
        &mut self,
        render_target: RenderTargetHandle,
        render_package: RenderPackage,
    ) -> RenderResult<()> {
        let RenderPackage {
            scene: scene_handle,
            pov,
        } = render_package;

        let scene = self.scenes.get_mut(&scene_handle).unwrap();

        scene.sync_frame_data(&self.meshes);

        let render_target = self
            .render_targets
            .get_mut(&render_target)
            .ok_or(RenderError::InvalidRenderTarget)?;

        let device = get_device();

        let _ = render_target.begin_rendering(device)?;

        let (indirect_buffer, indirect_iter) = scene.indirect_draw_iterator();

        render_target.start_composition_pass(device)?;

        render_target.end_rendering(device);

        render_target.prepare_transfer(device);

        render_target.submit_render(device)?;

        Ok(())
    }

    pub fn present_to_target(
        &mut self,
        present_target: PresentTargetHandle,
        render_target: RenderTargetHandle,
    ) -> PresentResult<()> {
        let device = get_device();

        unsafe { device.device_wait_idle()? }

        let present_target = self.present_targets.get_mut(&present_target).unwrap();
        let render_target = self.render_targets.get_mut(&render_target).unwrap();

        let PresentData {
            cmd_buffer,
            swapchain,
            image_acquired,
            image_ready,
            image_index: index,
        } = present_target
            .record_present(device, render_target.output_image())
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

            device.queue_submit(device.present_queue, &submits, vk::Fence::null())?;
        };

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .image_indices(&indices)
            .wait_semaphores(&images_ready);

        unsafe {
            self.swapchain_loader
                .queue_present(device.present_queue, &present_info)?
        };

        render_target.next_frame();

        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        let device = get_device();

        unsafe {
            let _ = device.device_wait_idle();
        }

        self.present_targets
            .drain()
            .for_each(|(_, mut target)| target.destroy(device));
    }
}
