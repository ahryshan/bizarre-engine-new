use std::{
    any::TypeId,
    collections::{BTreeMap, VecDeque},
};

use component_storage::{ComponentStorage, IntoStoredComponent, Storable, StoredComponent};
use error::{ComponentError, ComponentResult};

use crate::entity::Entity;

pub mod component_storage;
pub mod error;

/// A marker trait that must be implemented for all types used as components
pub trait Component: Storable {}

/// Type for storing all registered and added components inside a [`World`](crate::world::World).
#[derive(Default)]
pub struct Components {
    lookup: BTreeMap<TypeId, usize>,
    storages: Vec<ComponentStorage>,
    bitmasks: Vec<u128>,
    entity_bitmasks: Vec<(Entity, u128)>,
    storage_capacity: usize,
    id_dumpster: VecDeque<usize>,
}

impl Components {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<C: IntoStoredComponent>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> ComponentResult {
        let component = component.into_stored_component();
        let index = self.get_index_for_stored(&component)?;
        self.storages[index].insert(entity, component)?;

        let (stored_entity, stored_bitmask) = self.entity_bitmasks[entity.index()];

        let pair = if stored_entity == entity {
            (entity, stored_bitmask | self.bitmasks[index])
        } else {
            (entity, self.bitmasks[index])
        };

        self.entity_bitmasks[entity.index()] = pair;

        Ok(())
    }

    pub fn get<C: Component>(&self, entity: Entity) -> ComponentResult<&C> {
        let index = self.get_index::<C>()?;

        self.storages[index].get(entity)
    }

    pub fn get_mut<C: Component>(&self, entity: Entity) -> ComponentResult<&mut C> {
        let index = self.get_index::<C>()?;
        self.storages[index].get_mut(entity)
    }

    /// Removes a component for the entity returning the value if the component was previously
    /// assigned for entity
    pub fn remove<C: Component>(&mut self, entity: Entity) -> Option<C> {
        let index = self.get_index::<C>().ok()?;

        let component = self.storages[index].remove::<C>(entity);
        self.entity_bitmasks[entity.index()].1 ^= self.bitmasks[index];
        component
    }

    pub fn remove_entity(&mut self, mut entity: Entity) {
        for storage in self.storages.iter_mut() {
            storage.forget_entity(entity);
        }
        entity.clear_gen();
        self.entity_bitmasks[entity.index()] = (entity, 0);
    }

    pub fn get_storage<C: Component>(&self) -> ComponentResult<&ComponentStorage> {
        let index = self.get_index::<C>()?;
        Ok(&self.storages[index])
    }

    pub fn get_storage_mut<C: Component>(&mut self) -> ComponentResult<&mut ComponentStorage> {
        let index = self.get_index::<C>()?;
        Ok(&mut self.storages[index])
    }

    /// Expands all underlying storages by 1
    pub fn expand(&mut self) {
        self.expand_by(1);
    }

    /// Expands all underlying storages by number provided
    pub fn expand_by(&mut self, by: usize) {
        self.storage_capacity += by;
        self.entity_bitmasks.push((Entity::from_gen_id(0, 1), 0));

        for storage in self.storages.iter_mut() {
            storage.expand_by(by)
        }
    }

    /// Registers a component of type `C` with this `Components`
    ///
    /// If there it's possible will reuse some old index for the newly created
    /// [`ComponentStorage`], but if there is a `ComponentStorage` already present for the
    /// component, this function will do nothing
    ///
    pub fn register<C: Component>(&mut self) {
        if self.get_index::<C>().is_ok() {
            return;
        }

        if let Some(index) = self.id_dumpster.pop_front() {
            self.storages[index] = ComponentStorage::with_capacity::<C>(self.storage_capacity);
            self.bitmasks[index] = 1 << index;
            self.lookup.insert(C::inner_type_id(), index);
        } else {
            let index = self.storages.len();

            self.storages
                .push(ComponentStorage::with_capacity::<C>(self.storage_capacity));
            self.bitmasks.push(1 << index);

            self.lookup.insert(C::inner_type_id(), index);
        }
    }

    /// Removes a storage from the `Components`.
    ///
    /// Note that the storage won't be removed physically from the `Components`, but it's just
    /// will be 'forgotten', making it unaccessible for new queries and [`Components::get_storage`] or [Components::get_storage_mut]
    /// but it will be possible to get access to the underlying data through raw pointers, created before the
    /// `remove_storage` call. The removed storage data will still be present in the memory
    /// location until it gets reused for a new storage.
    ///
    /// If there is no storage for the `C` to begin with, it won't do anything
    ///
    pub fn unregister<C: Component>(&mut self) {
        let index = if let Ok(index) = self.get_index::<C>() {
            index
        } else {
            return;
        };

        self.id_dumpster.push_back(index);
        self.lookup.remove(&C::inner_type_id());
    }

    /// Gets index of the storage for components of type `C` if such exists in this `Components`.
    ///
    /// # Errors
    ///
    /// Returns [ComponentError::NotPresentStorage] if there is no storage for the `C`
    #[inline(always)]
    fn get_index<C: Component>(&self) -> ComponentResult<usize> {
        self.lookup
            .get(&C::inner_type_id())
            .copied()
            .ok_or(ComponentError::NotPresentStorage(C::inner_type_name()))
    }

    #[inline(always)]
    fn get_index_for_stored(&self, component: &StoredComponent) -> ComponentResult<usize> {
        self.lookup.get(&component.inner_type_id()).copied().ok_or(
            ComponentError::NotPresentStorage(component.component_name()),
        )
    }

    pub fn filter_entities(&self, type_ids: &[TypeId]) -> Vec<Entity> {
        let bitmask = type_ids.iter().fold(0, |acc, tid| {
            let comp_mask = self
                .lookup
                .get(tid)
                .map(|index| self.bitmasks[*index])
                .unwrap_or(0);

            acc | comp_mask
        });

        self.entity_bitmasks
            .iter()
            .filter_map(|(e, b)| {
                if b & bitmask == bitmask {
                    Some(*e)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Components {
    /// # Safety
    ///
    /// The `Components` instance must have a storage for the `C` type provided
    ///
    /// # Panics
    ///
    /// Absence of a storage for type `C` will result in panic
    pub unsafe fn get_index_unchecked<C: Component>(&self) -> usize {
        self.lookup[&C::inner_type_id()]
    }

    /// # Safety
    ///
    /// The `Components` instance must have a storage for the `C` type provided
    ///
    /// # Panics
    ///
    /// Absence of a storage for type `C` will result in panic
    pub unsafe fn get_storage_unchecked<C: Component>(&self) -> &ComponentStorage {
        let index = self.get_index_unchecked::<C>();
        &self.storages[index]
    }

    /// # Safety
    ///
    /// The `Components` instance must have a storage for the `C` type provided
    ///
    /// # Panics
    ///
    /// Absence of a storage for type `C` will result in panic
    pub unsafe fn get_storage_mut_unchecked<C: Component>(&mut self) -> &mut ComponentStorage {
        let index = self.get_index_unchecked::<C>();
        &mut self.storages[index]
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use anyhow::Result;

    use crate::{
        component::{component_storage::Storable, Component},
        entity::Entity,
        test_commons::Health,
    };

    use super::Components;

    #[test]
    fn should_register_component() {
        let mut components = Components::new();
        components.register::<Health>();

        assert!(components.storages.len() == 1);
        assert!(components.lookup.get(&TypeId::of::<Health>()) == Some(0).as_ref());
        assert!(components.storages[0].capacity() == 0);
        assert!(components.storages[0].occupied() == 0);
        assert!(components.bitmasks[0] == 1);
    }

    #[test]
    fn should_unregister_component() {
        let mut components = Components::new();
        components.register::<Health>();
        components.unregister::<Health>();

        assert!(!components.lookup.contains_key(&Health::inner_type_id()));
    }

    #[test]
    fn should_expand() {
        let mut components = Components::new();

        assert!(components.storage_capacity == 0);

        components.expand();

        assert!(components.storage_capacity == 1);

        components.expand_by(9);

        assert!(components.storage_capacity == 10);
    }

    #[test]
    fn should_insert_component() -> Result<()> {
        let mut components = Components::new();

        components.expand();
        components.register::<Health>();

        let entity = Entity::from_gen_id(1, 0);

        components.insert(entity, Health(100))?;

        assert!(components.storages[0].capacity() == 1);
        assert!(components.storages[0].occupied() == 1);
        assert!(components.entity_bitmasks[0].1 == 1);

        Ok(())
    }

    #[test]
    fn should_get_component() -> Result<()> {
        let mut storage = Components::new();

        storage.expand();
        storage.register::<Health>();

        let entity = Entity::from_gen_id(1, 0);

        storage.insert(entity, Health(100))?;

        let health: &Health = storage.get(entity)?;

        assert!(health == &Health(100));

        let cloned = health.clone();

        let health_mut: &mut Health = storage.get_mut(entity)?;

        assert!(&cloned == health_mut);

        Ok(())
    }

    #[test]
    fn should_remove_entity() -> Result<()> {
        let mut storage = Components::new();
        storage.expand();
        storage.register::<Health>();

        let entity = Entity::from_gen_id(1, 0);

        storage.insert(entity, Health(100))?;

        storage.remove_entity(entity);

        assert!(storage.storages[0].capacity() == 1);
        assert!(storage.storages[0].occupied() == 0);
        assert!(storage.entity_bitmasks[0].1 == 0);
        assert!(!storage.storages[0].has_entity(entity));

        Ok(())
    }
}
