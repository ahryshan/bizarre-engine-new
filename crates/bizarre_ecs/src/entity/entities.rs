use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    component_result::{ComponentError, ComponentResult},
    component_storage::ComponentStorage,
    entity_builder::EntityBuilder,
    query::Query,
    Entity,
};

#[derive(Default)]
pub struct Entities {
    pub(crate) entities: Vec<Entity>,
    pub(crate) components: HashMap<TypeId, ComponentStorage>,
    pub(crate) comp_bitmasks: HashMap<TypeId, u128>,
    pub(crate) entity_bitmasks: Vec<u128>,
    pub(crate) entity_count: usize,
    pub(crate) next_id: usize,
}

impl Entities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        let entity = Entity::new(id, 0);
        self.entity_count += 1;
        self.components.iter_mut().for_each(|(_, c)| c.push(None));
        self.entity_bitmasks.push(0);
        entity
    }

    pub fn register_component<T>(&mut self)
    where
        T: 'static,
    {
        if self.components.contains_key(&TypeId::of::<T>()) {
            return;
        }

        let k = TypeId::of::<T>();
        self.components
            .insert(k, vec![None; self.entity_count].into());
        self.comp_bitmasks.insert(k, 2 << self.comp_bitmasks.len());
    }

    pub fn insert_component<T>(&mut self, entity: Entity, component: T) -> ComponentResult<()>
    where
        T: 'static,
    {
        let storage = self
            .components
            .get_mut(&TypeId::of::<T>())
            .ok_or(ComponentError::not_registered::<T>())?;

        let _ = storage[entity.id()].insert(Rc::new(RefCell::new(component)));
        self.entity_bitmasks[entity.id().as_usize()] |= self.component_bitmask::<T>()?;

        Ok(())
    }

    pub fn get_storage<T>(&self) -> &ComponentStorage
    where
        T: 'static,
    {
        self.components.get(&TypeId::of::<T>()).unwrap()
    }

    pub fn spawn<'e, 'b>(&'e mut self) -> EntityBuilder<'b>
    where
        'e: 'b,
    {
        EntityBuilder {
            entity: self.create_entity(),
            entities: self,
        }
    }

    pub fn component_bitmask<T>(&self) -> ComponentResult<u128>
    where
        T: 'static,
    {
        self.comp_bitmasks
            .get(&TypeId::of::<T>())
            .copied()
            .ok_or(ComponentError::not_registered::<T>())
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::entity::{component_storage::ComponentStorage, EntityGen, EntityId};

    use super::Entities;

    struct Health(u32);
    struct Velocity(f64);
    struct Acceleration(f64);

    #[test]
    fn should_create_entity() {
        let mut entities = Entities::new();
        let entity = entities.create_entity();

        assert!(entity.id() == EntityId::from(0));
        assert!(entity.gen() == EntityGen::from(0));
    }

    #[test]
    fn should_entity_repository() {
        let entities = Entities::new();
        assert!(entities.entities.len() == 0);
        assert!(entities.components.len() == 0);
        assert!(entities.entity_count == 0);
    }

    #[test]
    fn should_register_components() {
        let mut entities = Entities::new();
        entities.register_component::<Health>();
        assert!(entities.components.len() == 1);
        let component_storage = entities.components.get(&TypeId::of::<Health>()).unwrap();
        assert!(component_storage.len() == 0);
    }

    #[test]
    fn should_insert_component() {
        let mut entities = Entities::new();
        entities.register_component::<Health>();
        let entity = entities.create_entity();
        entities.insert_component(entity, Health(100));
        let component_storage = entities.components.get(&TypeId::of::<Health>()).unwrap();
        assert!(component_storage.len() == 1);
        let retrieved_component = component_storage[entity.id()].as_ref().unwrap().borrow();
        let retrieved_component = retrieved_component.downcast_ref::<Health>().unwrap();

        assert!(retrieved_component.0 == 100);
    }

    fn map_storage(storage: &ComponentStorage) -> Vec<Option<()>> {
        storage
            .iter()
            .map(|opt| match opt {
                Some(_) => Some(()),
                None => None,
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn should_build_entity() {
        let mut entities = Entities::new();

        let entity = entities
            .spawn()
            .with_component(Health(100))
            .with_component(Velocity(128.0))
            .build();

        let entity1 = entities
            .spawn()
            .with_component(Health(200))
            .with_component(Acceleration(20.0))
            .build();

        let entity2 = entities.spawn().with_component(Velocity(1000.0)).build();

        let health_storage = entities.get_storage::<Health>();
        let velocity_storage = entities.get_storage::<Velocity>();
        let acceleration_storage = entities.get_storage::<Acceleration>();

        if let [Some(_), Some(_), None] = health_storage.as_slice() {
            // Empty here, because there is no if not let ...
        } else {
            panic!(
                "Expected health storage to be [Some, Some, None], got {:?} instead",
                map_storage(health_storage)
            )
        }

        if let [Some(_), None, Some(_)] = velocity_storage.as_slice() {
        } else {
            panic!(
                "Expected velocity storage to be [Some, None, Some], got {:?} instead",
                map_storage(velocity_storage)
            )
        }

        if let [None, Some(_), None] = acceleration_storage.as_slice() {
        } else {
            panic!(
                "Expected velocity storage to be [Some, None, Some], got {:?} instead",
                map_storage(acceleration_storage)
            )
        }
    }
}
