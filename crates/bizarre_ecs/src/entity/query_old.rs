use std::any::TypeId;

use super::{component_storage::Component, entities::Entities, EntityId};

pub struct Query<'a> {
    entities: &'a Entities,
    map: u128,
    type_ids: Vec<TypeId>,
}

impl<'q> Query<'q> {
    pub fn new(entities: &'q Entities) -> Self {
        Self {
            map: 0,
            entities,
            type_ids: Default::default(),
        }
    }

    pub fn with_component<T>(&mut self) -> &mut Self
    where
        T: 'static,
    {
        self.map |= self.entities.component_bitmask::<T>();
        self.type_ids.push(TypeId::of::<T>());
        self
    }

    pub fn run<'a>(&self) -> Vec<Vec<Component>> {
        let ids = self
            .entities
            .entity_bitmasks
            .iter()
            .enumerate()
            .filter_map(|(ent, ent_map)| {
                if ent_map & self.map == self.map {
                    Some(ent)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let storages = self
            .type_ids
            .iter()
            .map(|id| {
                self.entities
                    .components
                    .get(id)
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, component)| {
                        if !ids.contains(&i) {
                            return None;
                        }

                        Some(component.as_ref().unwrap().clone())
                    })
                    .collect()
            })
            .collect();

        storages
    }
}
