use std::{collections::HashMap, ffi::c_void, path::Path};

use std::fmt::Debug;

use ash::vk::{self, Handle as _};
use bizarre_core::{
    handle::{DenseHandleStrategy, HandlePlacement, HandleStrategy, SparseHandleStrategy},
    Handle,
};
use bizarre_ecs::prelude::Resource;
use bizarre_log::core_info;
use bizarre_window::Window;
use nalgebra_glm::UVec2;

use crate::antialiasing::Antialiasing;
use crate::material::pipeline::{VulkanPipeline, VulkanPipelineRequirements};
use crate::scene::SceneHandle;
use crate::{
    material::{
        material_instance::{MaterialInstance, MaterialInstanceHandle},
        Material, MaterialHandle,
    },
    mesh::{Mesh, MeshHandle},
    present_target::{PresentTarget, PresentTargetHandle},
    render_target::{RenderTargetHandle, SwapchainRenderTarget},
    scene::Scene,
    vulkan_context::{get_device, get_instance},
};

#[derive(Resource)]
pub struct AssetStore<A: 'static, S: 'static> {
    data: HashMap<Handle<A>, A>,
    handle_strategy: S,
}

impl<A, S: Default> Default for AssetStore<A, S> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            handle_strategy: Default::default(),
        }
    }
}

impl<A, S: HandleStrategy<A> + Default> AssetStore<A, S> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<A, S: HandleStrategy<A>> AssetStore<A, S> {
    pub fn with_strategy(handle_strategy: S) -> Self {
        Self {
            handle_strategy,
            data: Default::default(),
        }
    }

    pub fn insert(&mut self, asset: A) -> Handle<A> {
        let handle = self.handle_strategy.new_handle(&asset);
        self.data.insert(handle, asset);
        handle
    }

    pub fn contains(&self, handle: Handle<A>) -> HandlePlacement {
        self.handle_strategy.handle_placement(&handle)
    }

    pub fn delete(&mut self, handle: Handle<A>) -> Option<A> {
        self.handle_strategy.mark_deleted(handle);
        self.data.remove(&handle)
    }

    pub fn get(&self, handle: &Handle<A>) -> Option<&A> {
        self.data.get(handle)
    }

    pub fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A> {
        self.data.get_mut(handle)
    }
}

pub type DenseAssetStore<T> = AssetStore<T, DenseHandleStrategy<T>>;
pub type SparseAssetStore<T> = AssetStore<T, SparseHandleStrategy<T>>;

#[derive(Default, Resource)]
pub struct RenderAssets {
    pub render_targets: DenseAssetStore<SwapchainRenderTarget>,
    pub present_targets: SparseAssetStore<PresentTarget>,
    pub meshes: DenseAssetStore<Mesh>,
    pub materials: DenseAssetStore<Material>,
    pub material_instances: DenseAssetStore<MaterialInstance>,
    pub scenes: DenseAssetStore<Scene>,
}

impl RenderAssets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_material(
        &mut self,
        pipeline_requirements: VulkanPipelineRequirements,
    ) -> MaterialHandle {
        let pipeline =
            VulkanPipeline::from_requirements(&pipeline_requirements, None, get_device()).unwrap();

        let material = Material::new(pipeline, pipeline_requirements.bindings);

        let handle = self.materials.insert(material);

        handle
    }

    pub fn insert_material(&mut self, material: Material) -> MaterialHandle {
        self.materials.insert(material)
    }

    pub fn create_material_instance(
        &mut self,
        material_handle: MaterialHandle,
    ) -> Option<(MaterialInstanceHandle, &mut MaterialInstance)> {
        let material = self.materials.get(&material_handle)?;
        let instance = MaterialInstance::new(material_handle, material).ok()?;

        let handle = self.material_instances.insert(instance);
        let instance = self.material_instances.get_mut(&handle).unwrap();

        Some((handle, instance))
    }

    pub fn material_with_instance(
        &self,
        instance_handle: &MaterialInstanceHandle,
    ) -> Option<(&Material, &MaterialInstance)> {
        let instance = self.material_instances.get(instance_handle)?;
        let material = self.materials.get(&instance.material_handle)?;

        Some((material, instance))
    }

    pub fn load_mesh<P>(&mut self, path: P) -> MeshHandle
    where
        P: AsRef<Path> + Debug,
    {
        let mesh = Mesh::load_from_obj(path);
        self.meshes.insert(mesh)
    }

    pub fn create_present_target2(
        &mut self,
        window: &bizarre_sdl::window::Window,
        image_count: u32,
    ) -> PresentTargetHandle {
        let instance_handle = get_instance().handle().as_raw() as usize;
        let surface = window.vulkan_create_surface(instance_handle).unwrap();
        let surface = vk::SurfaceKHR::from_raw(surface);

        let present_target = PresentTarget::new2(
            get_device().cmd_pool,
            image_count,
            surface,
            window.id() as usize,
        )
        .unwrap();

        self.present_targets.insert(present_target)
    }

    pub fn create_present_target(
        &mut self,
        window: &Window,
        image_count: u32,
    ) -> PresentTargetHandle {
        let device = get_device();

        let data = unsafe {
            let display = bizarre_window::get_wayland_display_ptr() as *mut vk::wl_display;
            let surface = window.raw_window_ptr() as *mut c_void;

            PresentTarget::new(
                device.cmd_pool,
                image_count,
                window.size(),
                display,
                surface,
                window.handle().as_raw(),
            )
            .unwrap()
        };

        let handle = self.present_targets.insert(data);

        handle
    }

    pub fn present_target_mut(
        &mut self,
        handle: &PresentTargetHandle,
    ) -> Option<&mut PresentTarget> {
        self.present_targets.get_mut(handle)
    }

    pub fn create_swapchain_render_target(
        &mut self,
        extent: UVec2,
        image_count: u32,
        antialiasing: Antialiasing,
    ) -> RenderTargetHandle {
        let device = get_device();

        let render_target = SwapchainRenderTarget::new(
            device,
            extent,
            device.cmd_pool,
            antialiasing.into(),
            image_count,
        )
        .unwrap();

        let handle = self.render_targets.insert(render_target);

        handle
    }

    pub fn create_scene(&mut self, image_count: u32) -> SceneHandle {
        self.scenes
            .insert(Scene::new(image_count as usize).unwrap())
    }

    pub fn scene_mut(&mut self, handle: &SceneHandle) -> Option<&mut Scene> {
        self.scenes.get_mut(handle)
    }
}
