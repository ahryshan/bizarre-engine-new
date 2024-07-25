use std::{
    any::TypeId,
    collections::{btree_map, BTreeMap},
};

use error::{ResourceError, ResourceResult};

use crate::component::component_storage::{IntoStoredComponent, Storable, StoredComponent};

pub mod error;

#[derive(Default)]
pub struct Resources {
    map: BTreeMap<TypeId, StoredComponent>,
}

/// A marker trait that must be implemented for all types used as resources
pub trait Resource: Storable {}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<R: IntoStoredComponent + Resource>(&mut self, resource: R) -> ResourceResult {
        let resource = resource.into_stored_component();

        if let btree_map::Entry::Vacant(e) = self.map.entry(resource.inner_type_id()) {
            e.insert(resource);
            Ok(())
        } else {
            Err(ResourceError::AlreadyPresent(resource.component_name()))
        }
    }

    pub fn get<R: Resource>(&self) -> ResourceResult<&R> {
        match self.map.get(&R::inner_type_id()) {
            Some(r) => Ok(r.downcast_ref().unwrap()),
            None => Err(ResourceError::NotPresent(R::inner_type_name())),
        }
    }

    pub fn get_mut<R: Resource>(&self) -> ResourceResult<&mut R> {
        match self.map.get(&R::inner_type_id()) {
            Some(r) => Ok(r.downcast_mut().unwrap()),
            None => Err(ResourceError::NotPresent(R::inner_type_name())),
        }
    }

    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.map
            .remove(&R::inner_type_id())
            .map(|r| unsafe { r.into_inner() })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};

    use crate::test_commons::Motd;

    use super::{error::ResourceError, Resources};

    #[test]
    fn should_insert_resource() -> Result<()> {
        let mut storage = Resources::new();

        storage.insert(Motd("Hello, World!"))?;

        Ok(())
    }

    #[test]
    fn should_err_on_double_insert() -> Result<()> {
        let mut storage = Resources::new();

        storage.insert(Motd("Hello, World!"))?;
        match storage.insert(Motd("Hello, World!")) {
            Err(ResourceError::AlreadyPresent(_)) => Ok(()),
            _ => Err(anyhow!(
                "Expected resource storage to prevent double insert"
            )),
        }
    }

    #[test]
    fn should_get_resource() -> Result<()> {
        let mut storage = Resources::new();
        storage.insert(Motd("Hello, World!"))?;

        let health = storage.get::<Motd>()?;

        assert!(health == &Motd("Hello, World!"));

        Ok(())
    }

    #[test]
    fn should_get_resource_mut() -> Result<()> {
        let mut r = Resources::new();
        r.insert(Motd("hello world"))?;

        let motd = r.get_mut::<Motd>()?;

        motd.0 = "Hello, World!";
        let cloned = motd.clone();

        let health = r.get::<Motd>()?;

        assert!(health == &cloned);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn should_not_get_nonexistent_resource() {
        let r = Resources::new();
        r.get::<Motd>().unwrap();
    }
}
