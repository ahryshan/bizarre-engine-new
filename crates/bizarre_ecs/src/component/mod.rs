use std::collections::{BTreeMap, VecDeque};

use crate::{
    entity::Entity,
    resource::{ComponentBuffer, Resource, ResourceId},
};

pub trait Component: Resource {}

impl<T: Resource> Component for T {}

pub struct ComponentRegistry {
    storages: Vec<Option<ComponentBuffer>>,
    capacity: usize,
    lookup: BTreeMap<ResourceId, usize>,
    index_dumpster: VecDeque<usize>,
    frozen_entities: Vec<Entity>,
}

impl ComponentRegistry {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storages: Default::default(),
            capacity,
            lookup: Default::default(),
            index_dumpster: Default::default(),
            frozen_entities: vec![Entity::from_gen_id(0, 0); capacity],
        }
    }

    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn expand_by(&mut self, by: usize) {
        self.capacity += by;

        self.storages
            .iter_mut()
            .flatten()
            .for_each(|b| b.expand_by(by));
    }

    pub fn expand(&mut self) {
        self.expand_by(1);
    }

    pub fn storage<T: Component>(&self) -> Option<&ComponentBuffer> {
        let index = self.index::<T>()?;

        self.storages[index].as_ref()
    }

    pub fn storage_mut<T: Component>(&mut self) -> Option<&mut ComponentBuffer> {
        let index = self.index::<T>()?;

        self.storages[index].as_mut()
    }

    pub fn component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.has_entity(entity) {
            return None;
        }

        let storage = self.storage::<T>()?;

        storage.get(entity.index())
    }

    pub fn component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.has_entity(entity) {
            return None;
        }

        let storage = self.storage_mut::<T>()?;

        storage.get_mut(entity.index())
    }

    pub fn register<T: Component>(&mut self) {
        if self.index::<T>().is_some() {
            return;
        }

        let new_storage = ComponentBuffer::with_capacity::<T>(self.capacity);
        let index = if let Some(index) = self.index_dumpster.pop_front() {
            self.storages[index] = Some(new_storage);
            index
        } else {
            let index = self.storages.len();
            self.storages.push(Some(new_storage));
            index
        };

        self.lookup.insert(T::id(), index);
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> Option<T> {
        let index = self
            .index::<T>()
            .unwrap_or_else(|| panic!("Component `{}` is not registered", T::name()));

        self.frozen_entities[entity.index()] = entity;

        self.storages[index]
            .as_mut()
            .unwrap()
            .insert(entity.index(), component)
    }

    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if self.frozen_entities[entity.index()] != entity {
            return None;
        }

        let index = self.index::<T>()?;

        self.storages[index]
            .as_mut()
            .unwrap()
            .remove::<T>(entity.index())
    }

    pub fn remove_storage<T: Component>(&mut self) -> Option<ComponentBuffer> {
        let index = self.index::<T>()?;

        let ret = self.storages[index].take();
        self.lookup.remove(&T::id());
        ret
    }

    pub fn has_entity(&self, entity: Entity) -> bool {
        self.frozen_entities[entity.index()] == entity
    }

    pub fn has_storage<T: Component>(&self) -> bool {
        self.index::<T>().is_some()
    }

    pub fn has_storage_for_id(&self, id: &ResourceId) -> bool {
        self.index_by_id(id).is_some()
    }

    pub fn has_component<T: Component>(&self) -> bool {
        self.index::<T>().is_some()
    }

    pub fn has_component_by_id(&self, id: &ResourceId) -> bool {
        self.index_by_id(id).is_some()
    }

    pub fn has_component_for_entity<T: Component>(&self, entity: Entity) -> bool {
        self.has_entity(entity)
            && self
                .storage::<T>()
                .map(|s| s.is_valid(entity.index()))
                .unwrap_or(false)
    }

    pub fn has_component_for_entity_by_id(&self, entity: Entity, id: &ResourceId) -> bool {
        let index = self.index_by_id(id);
        if index.is_none() {
            return false;
        }

        let index = index.unwrap();

        self.has_entity(entity)
            && self.storages[index]
                .as_ref()
                .map(|s| s.is_valid(entity.index()))
                .unwrap_or(false)
    }

    fn index_by_id(&self, id: &ResourceId) -> Option<usize> {
        self.lookup.get(id).copied()
    }

    fn index<T: Component>(&self) -> Option<usize> {
        self.lookup.get(&T::id()).copied()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {

    use crate::entity::Entity;

    use super::ComponentRegistry;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Health(pub u32);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Mana(pub u32);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Name(pub &'static str);

    #[test]
    #[should_panic]
    pub fn should_panic_on_unregistered_insert() {
        let mut components = ComponentRegistry::new();
        let entity = Entity::from_gen_id(1, 0);
        components.insert(entity, Health(100));
    }

    #[test]
    pub fn should_register_components() {
        let mut c = ComponentRegistry::new();
        c.register::<Health>();
        c.register::<Mana>();
        c.register::<Name>();

        assert!(c.capacity == 0);
        assert!(c.storages.len() == 3);
    }

    #[test]
    pub fn should_insert_components() {
        let mut c = ComponentRegistry::with_capacity(3);
        let entity_0 = Entity::from_gen_id(1, 0);

        c.register::<Health>();
        c.register::<Mana>();
        c.register::<Name>();

        if c.insert(entity_0, Health(100)).is_some() {
            panic!("There must be no Health for {entity_0:?}");
        };

        assert!(c.has_entity(entity_0));
        assert!(c.has_component_for_entity::<Health>(entity_0));
        assert!(c.component(entity_0) == Some(&Health(100)));
    }
}
