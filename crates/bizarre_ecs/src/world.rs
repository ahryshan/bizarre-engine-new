use crate::resource::{
    registry::{ResourceReadLock, ResourceRegistry, ResourceResult, ResourceWriteLock},
    resource::IntoResource,
};

#[derive(Default)]
pub struct World {
    resources: ResourceRegistry,
}

impl World {
    pub fn get_resource<T>(&self) -> ResourceResult<ResourceReadLock<T>>
    where
        T: 'static,
    {
        self.resources.get()
    }

    pub fn get_resource_mut<T>(&self) -> ResourceResult<ResourceWriteLock<T>>
    where
        T: 'static,
    {
        self.resources.get_mut()
    }

    pub fn with_resource<T>(&self, closure: impl Fn(&T) -> ()) -> ResourceResult<()>
    where
        T: 'static,
    {
        self.resources.with_resource(closure)
    }

    pub fn with_resource_mut<T>(&self, closure: impl Fn(&mut T) -> ()) -> ResourceResult<()>
    where
        T: 'static,
    {
        self.resources.with_resource_mut(closure)
    }

    pub fn insert_resource<T>(&mut self, resource: T) -> ResourceResult<()>
    where
        T: 'static + IntoResource,
    {
        self.resources.insert(resource)
    }

    pub fn remove_resource<T>(&mut self) -> ResourceResult<T>
    where
        T: 'static,
    {
        self.resources.remove()
    }
}
