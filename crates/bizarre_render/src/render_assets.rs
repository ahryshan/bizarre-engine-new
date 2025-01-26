use std::{collections::HashMap, ffi::c_void, path::Path};

use std::fmt::Debug;

use ash::vk::{self, Handle as _};
use bizarre_core::handle::IntoHandle;
use bizarre_core::{
    handle::{DenseHandleStrategy, HandlePlacement, HandleStrategy, SparseHandleStrategy},
    Handle,
};
use bizarre_ecs::prelude::Resource;
use bizarre_log::core_info;
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

pub trait AssetStore<A, S: HandleStrategy<A>> {
    fn insert(&mut self, asset: A) -> Handle<A>;

    fn contains(&self, handle: &Handle<A>) -> HandlePlacement;

    fn remove(&mut self, handle: Handle<A>) -> Option<A>;

    fn get(&self, handle: &Handle<A>) -> Option<&A>;

    fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A>;
}

pub struct SparseAssetStore<A> {
    data: HashMap<Handle<A>, A>,
    handle_strategy: SparseHandleStrategy<A>,
}

impl<T> Default for SparseAssetStore<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            handle_strategy: Default::default(),
        }
    }
}

impl<A> SparseAssetStore<A> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<A: IntoHandle> AssetStore<A, SparseHandleStrategy<A>> for SparseAssetStore<A> {
    fn insert(&mut self, asset: A) -> Handle<A> {
        let (handle, _) = self.handle_strategy.new_handle(&asset);
        self.data.insert(handle, asset);
        handle
    }

    fn contains(&self, handle: &Handle<A>) -> HandlePlacement {
        self.handle_strategy.handle_placement(&handle)
    }

    fn remove(&mut self, handle: Handle<A>) -> Option<A> {
        self.handle_strategy.mark_deleted(handle);
        self.data.remove(&handle)
    }

    fn get(&self, handle: &Handle<A>) -> Option<&A> {
        self.data.get(handle)
    }

    fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A> {
        self.data.get_mut(handle)
    }
}

pub struct DenseAssetStore<T> {
    data: Vec<Option<T>>,
    handle_strategy: DenseHandleStrategy<T>,
}

impl<T> Default for DenseAssetStore<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            handle_strategy: Default::default(),
        }
    }
}

impl<T> AssetStore<T, DenseHandleStrategy<T>> for DenseAssetStore<T> {
    fn insert(&mut self, asset: T) -> Handle<T> {
        let (handle, reused) = self.handle_strategy.new_handle(&asset);

        if reused {
            self.data[handle.as_raw()] = Some(asset);
        } else {
            self.data.push(Some(asset))
        }

        handle
    }

    fn contains(&self, handle: &Handle<T>) -> HandlePlacement {
        self.handle_strategy.handle_placement(&handle)
    }

    fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let value = match self.contains(&handle) {
            HandlePlacement::Present => self.data[handle.as_raw()].take(),
            _ => None,
        };
        self.handle_strategy.mark_deleted(handle);
        value
    }

    fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.data.get(handle.as_raw())?.as_ref()
    }

    fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.data.get_mut(handle.as_raw())?.as_mut()
    }
}

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
        pipeline_requirements: &VulkanPipelineRequirements,
    ) -> MaterialHandle {
        let pipeline =
            VulkanPipeline::from_requirements(pipeline_requirements, None, get_device()).unwrap();

        let material = Material::new(pipeline, &pipeline_requirements.bindings);

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
