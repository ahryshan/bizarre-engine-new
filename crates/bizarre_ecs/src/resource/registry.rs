use std::{
    any::TypeId,
    collections::{BTreeMap, VecDeque},
    sync::{
        MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

use super::resource::{IntoResource, RegisteredResource, ResourceError};

#[derive(Default)]
pub struct ResourceRegistry {
    resources: Vec<Option<RwLock<RegisteredResource>>>,
    type_map: BTreeMap<TypeId, usize>,
    index_dumpster: VecDeque<usize>,
}

pub type ResourceReadLock<'a, T> = MappedRwLockReadGuard<'a, T>;
pub type ResourceWriteLock<'a, T> = MappedRwLockWriteGuard<'a, T>;
pub type ResourceResult<R> = Result<R, ResourceError>;

impl ResourceRegistry {
    pub fn insert<T>(&mut self, resource: T) -> ResourceResult<()>
    where
        T: IntoResource + 'static,
    {
        let type_id = TypeId::of::<T>();
        if self.type_map.contains_key(&type_id) {
            Err(ResourceError::already_present::<T>())
        } else {
            let index = if let Some(index) = self.index_dumpster.pop_front() {
                let removed = self.resources[index].take();
                drop(removed);
                self.resources[index] = Some(RwLock::new(resource.into_resource()));
                index
            } else {
                let index = self.resources.len();
                self.resources
                    .push(Some(RwLock::new(resource.into_resource())));
                index
            };
            self.type_map.insert(type_id, index);
            Ok(())
        }
    }

    pub fn remove<T>(&mut self) -> ResourceResult<T>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        match self.type_map.remove(&type_id) {
            Some(index) => {
                self.index_dumpster.push_back(index);
                let res = self.resources[index]
                    .take()
                    .unwrap()
                    .into_inner()
                    .unwrap()
                    .into_inner();

                Ok(res)
            }
            None => Err(ResourceError::not_present::<T>()),
        }
    }

    pub fn get<T>(&self) -> ResourceResult<ResourceReadLock<T>>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();

        let index = *self
            .type_map
            .get(&type_id)
            .ok_or(ResourceError::not_present::<T>())?;

        let lock = self.resources[index]
            .as_ref()
            .ok_or(ResourceError::not_present::<T>())?;

        let guard = lock.read().unwrap();

        let guard = RwLockReadGuard::try_map(guard, |r| r.as_ref().ok());

        match guard {
            Err(_) => Err(ResourceError::cannot_convert::<T>(&lock.read().unwrap())),
            Ok(guard) => Ok(guard),
        }
    }

    pub fn get_mut<T>(&self) -> ResourceResult<ResourceWriteLock<T>>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();

        let index = *self
            .type_map
            .get(&type_id)
            .ok_or(ResourceError::not_present::<T>())?;

        let lock = self.resources[index]
            .as_ref()
            .ok_or(ResourceError::not_present::<T>())?;

        let guard = lock.write().unwrap();

        let guard = RwLockWriteGuard::try_map(guard, |r| r.as_mut().ok());

        match guard {
            Err(_) => Err(ResourceError::cannot_convert::<T>(&lock.read().unwrap())),
            Ok(guard) => Ok(guard),
        }
    }

    pub fn with_resource<T, F>(&self, func: F) -> ResourceResult<()>
    where
        T: 'static,
        F: FnOnce(&T) -> (),
    {
        let resource = self.get()?;
        func(&resource);
        Ok(())
    }

    pub fn with_resource_mut<T, F>(&self, func: F) -> ResourceResult<()>
    where
        T: 'static,
        F: FnOnce(&mut T) -> (),
    {
        let mut resource = self.get_mut::<T>()?;
        func(&mut resource);
        Ok(())
    }
}
