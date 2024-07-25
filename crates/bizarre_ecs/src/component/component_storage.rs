use std::{
    any::{type_name, TypeId},
    ptr,
};

use crate::entity::Entity;

use super::{
    error::{ComponentError, ComponentResult},
    Component,
};

pub struct StoredComponent {
    type_id: TypeId,
    name: &'static str,
    data: *mut (),
}

pub trait Storable: 'static {
    fn inner_type_id() -> TypeId;
    fn inner_type_name() -> &'static str;
}

pub auto trait StorableAutoMarker {}
impl !StorableAutoMarker for StoredComponent {}

impl<T> Storable for T
where
    T: 'static + StorableAutoMarker,
{
    fn inner_type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn inner_type_name() -> &'static str {
        type_name::<T>()
    }
}

impl StoredComponent {
    pub fn downcast_ref<C: Storable>(&self) -> Option<&C> {
        if C::inner_type_id() != self.type_id {
            None
        } else {
            let r = unsafe { &*self.data.cast() };
            Some(r)
        }
    }

    pub fn downcast_mut<C: Storable>(&self) -> Option<&mut C> {
        if C::inner_type_id() != self.type_id {
            None
        } else {
            let r = unsafe { &mut *self.data.cast() };
            Some(r)
        }
    }

    /// Returns the underlying value converted to `C`
    ///
    /// # Safety
    ///
    /// This function assumes all the safety measures for [`core::ptr::read`].
    /// This function must be called with the same `C` as the StoredComponent was created from
    ///
    /// # Panics
    ///
    /// If the `C` provided is different from the type the `StoredComponent` was with this function
    /// will panic
    ///
    pub unsafe fn into_inner<C: Storable>(self) -> C {
        if C::inner_type_id() != self.inner_type_id() {
            panic!(
                "Trying to convert a StoredComponent of type `{}` into type `{}`",
                C::inner_type_name(),
                self.name
            );
        }
        ptr::read(self.data.cast())
    }

    pub fn inner_type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn component_name(&self) -> &'static str {
        self.name
    }
}

pub trait IntoStoredComponent {
    fn into_stored_component(self) -> StoredComponent;
}

impl<S: Storable> IntoStoredComponent for S {
    fn into_stored_component(self) -> StoredComponent {
        let data = {
            let boxed = Box::new(self);
            Box::into_raw(boxed) as *mut _
        };
        StoredComponent {
            data,
            name: S::inner_type_name(),
            type_id: S::inner_type_id(),
        }
    }
}

impl IntoStoredComponent for StoredComponent {
    fn into_stored_component(self) -> StoredComponent {
        self
    }
}

pub struct ComponentStorage {
    components: Vec<Option<StoredComponent>>,
    component_name: &'static str,
    type_id: TypeId,
    frozen_entities: Vec<Entity>,
    capacity: usize,
    occupied: usize,
}

#[allow(dead_code)]
impl ComponentStorage {
    pub fn new<C: Component>() -> Self {
        Self {
            components: Vec::new(),
            component_name: C::inner_type_name(),
            type_id: TypeId::of::<C>(),
            frozen_entities: Default::default(),
            capacity: 0,
            occupied: 0,
        }
    }

    pub fn with_capacity<C: Component>(capacity: usize) -> Self {
        let components = (0..capacity).map(|_| None).collect();
        let frozen_entities = vec![Entity::from_gen_id(0, 0); capacity];
        Self {
            components,
            component_name: C::inner_type_name(),
            type_id: TypeId::of::<C>(),
            frozen_entities,
            capacity,
            occupied: 0,
        }
    }

    /// Returns how much entities are registered to work with this storage
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns how much entities has a component in this storage
    pub fn occupied(&self) -> usize {
        self.occupied
    }

    pub fn expand(&mut self) {
        self.expand_by(1);
    }

    pub fn expand_by(&mut self, by: usize) {
        self.components.append(&mut (0..by).map(|_| None).collect());
        self.frozen_entities
            .append(&mut vec![Entity::from_gen_id(0, 0); by]);

        self.capacity += by;
    }

    pub fn insert<C: IntoStoredComponent>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> ComponentResult {
        let component = component.into_stored_component();
        self.check_stored(&component)?;

        if let Some(wrapped) = self.components.get_mut(entity.index()) {
            if wrapped.is_some() {
                let frozen = self.frozen_entities[entity.index()];
                if frozen == entity {
                    return Err(ComponentError::AlreadyPresentForEntity(
                        entity,
                        self.component_name,
                    ));
                }
            }

            *wrapped = Some(component);
            self.frozen_entities[entity.index()] = entity;
            self.occupied += 1;
            Ok(())
        } else {
            Err(ComponentError::OutOfBounds {
                index: entity.index(),
                len: self.capacity,
            })
        }
    }

    pub fn has_entity(&mut self, entity: Entity) -> bool {
        self.frozen_entities.contains(&entity)
    }

    pub fn forget_entity(&mut self, entity: Entity) {
        if self.has_entity(entity) {
            self.occupied -= 1;
            self.frozen_entities[entity.index()].clear_gen();
        }
    }

    pub fn remove<C: Component>(&mut self, entity: Entity) -> Option<C> {
        if self.has_entity(entity) {
            self.occupied -= 1;
            self.frozen_entities[entity.index()].clear_gen();
            self.components[entity.index()]
                .take()
                .map(|r| unsafe { r.into_inner() })
        } else {
            None
        }
    }

    pub(crate) fn get_raw(&self, entity: Entity) -> ComponentResult<&StoredComponent> {
        if let Some(wrapped) = self.components.get(entity.index()) {
            if wrapped.is_none() {
                Err(ComponentError::NotPresentForEntity(
                    entity,
                    self.component_name,
                ))
            } else {
                let frozen = self.frozen_entities[entity.index()];
                if frozen != entity {
                    return Err(ComponentError::NotPresentForEntity(
                        entity,
                        self.component_name,
                    ));
                }

                let r = wrapped.as_ref().unwrap();
                Ok(r)
            }
        } else {
            Err(ComponentError::OutOfBounds {
                index: entity.index(),
                len: self.components.len(),
            })
        }
    }

    pub(crate) fn get_raw_mut(&mut self, entity: Entity) -> ComponentResult<&mut StoredComponent> {
        if let Some(wrapped) = self.components.get_mut(entity.index()) {
            if wrapped.is_none() {
                Err(ComponentError::NotPresentForEntity(
                    entity,
                    self.component_name,
                ))
            } else {
                let frozen = self.frozen_entities[entity.index()];
                if frozen != entity {
                    return Err(ComponentError::NotPresentForEntity(
                        entity,
                        self.component_name,
                    ));
                }

                let r = wrapped.as_mut().unwrap();
                Ok(r)
            }
        } else {
            Err(ComponentError::OutOfBounds {
                index: entity.index(),
                len: self.frozen_entities.len(),
            })
        }
    }

    pub fn get<C: Component>(&self, entity: Entity) -> ComponentResult<&C> {
        self.check_type_id::<C>()?;

        let component = self.get_raw(entity)?;

        Ok(component.downcast_ref().unwrap())
    }

    pub fn get_mut<C: Component>(&self, entity: Entity) -> ComponentResult<&mut C> {
        self.check_type_id::<C>()?;

        let component = self.get_raw(entity)?;

        Ok(component.downcast_mut().unwrap())
    }

    #[inline(always)]
    fn check_stored(&self, component: &StoredComponent) -> ComponentResult {
        if component.inner_type_id() != self.type_id {
            Err(ComponentError::WrongType {
                expected: self.component_name,
                found: component.component_name(),
            })
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn check_type_id<C: Component>(&self) -> ComponentResult {
        if C::inner_type_id() != self.type_id {
            Err(ComponentError::WrongType {
                expected: self.component_name,
                found: C::inner_type_name(),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use anyhow::{anyhow, Result};

    use crate::{
        component::{component_storage::IntoStoredComponent, error::ComponentError, Component},
        entity::Entity,
        test_commons::{Health, Mana},
    };

    use super::ComponentStorage;

    #[test]
    fn should_insert_component() -> Result<()> {
        let (mut storage, entity) = setup_storage();

        storage.insert(entity, Health(100))?;

        Ok(())
    }

    #[test]
    fn should_fail_on_wrong_component() {
        let (mut storage, entity) = setup_storage();

        let err = storage.insert(entity, Mana(100)).err();

        if let Some(ComponentError::WrongType { .. }) = err {
        } else if let Some(err) = err {
            panic!("Expected to get a `ComponentError::WrongType` when inserting `Mana` component into a `Health` storage. Got instead: {err:?}");
        } else {
            panic!(
                "Expected to get an error when inserting `Mana` component into a `Health` storage"
            );
        }
    }

    #[test]
    fn should_fail_on_out_of_bounds() {
        let mut storage = ComponentStorage::new::<Health>();

        let entity = Entity::from_gen_id(0, 0);

        let err = storage.insert(entity, Health(100)).err();

        if let Some(ComponentError::OutOfBounds { .. }) = err {
        } else if let Some(err) = err {
            panic!("Expected to get a `ComponentError::OutOfBounds` when inserting a component in a storage with len = 0. Got instead: {err:?}");
        } else {
            panic!("Expected to get an error when inserting a component in a storage with len = 0");
        }
    }

    #[test]
    fn should_get_raw_component() -> Result<()> {
        let (mut storage, entity) = setup_storage();

        storage.insert(entity, Health(100))?;

        let component = storage.get_raw(entity)?;

        assert!(component.type_id == TypeId::of::<Health>());

        let health: &Health = component.downcast_ref().ok_or(anyhow!(
            "Couldn't downcast retrieved component into a Health"
        ))?;

        assert!(health.0 == 100);

        Ok(())
    }

    #[test]
    fn should_get_component() -> Result<()> {
        let (mut storage, entity) = setup_storage();

        storage.insert(entity, Health(100))?;

        let component: &Health = storage.get(entity)?;

        assert!(component.0 == 100);

        Ok(())
    }

    #[test]
    fn should_get_component_mut() -> Result<()> {
        let (mut storage, entity) = setup_storage();

        storage.insert(entity, Health(100))?;
        let clonned = {
            let component: &mut Health = storage.get_mut(entity)?;

            component.0 = 25;
            component.clone()
        };

        let component: &Health = storage.get(entity)?;

        assert!(component == &clonned, "Expected retrieved health to be equal to clonned health(retrieved: {component:?}, clonned: {clonned:?})");

        Ok(())
    }

    #[test]
    fn should_expand_storage() {
        let mut storage = ComponentStorage::new::<Health>();

        assert!(storage.capacity() == 0);
        assert!(storage.occupied() == 0);
        assert!(storage.frozen_entities.is_empty());
        assert!(storage.components.is_empty());

        storage.expand();

        assert!(storage.capacity() == 1);
        assert!(storage.occupied() == 0);
        assert!(storage.frozen_entities.len() == 1);
        assert!(storage.components.len() == 1);

        storage.expand_by(9);

        assert!(storage.capacity() == 10);
        assert!(storage.occupied() == 0);
        assert!(storage.frozen_entities.len() == 10);
        assert!(storage.components.len() == 10);
    }

    fn setup_storage() -> (ComponentStorage, Entity) {
        let storage = ComponentStorage::with_capacity::<Health>(1);

        let entity = Entity::from_gen_id(1, 0);
        (storage, entity)
    }
}
