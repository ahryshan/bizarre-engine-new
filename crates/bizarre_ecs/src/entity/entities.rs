use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    error::{EntityError, EntityResult},
    Entity,
};

#[derive(Default)]
pub struct Entities {
    entities: Vec<Entity>,
    id_dumpster: VecDeque<Entity>,
    next_id: AtomicU64,
}

impl Entities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        if entity.index() >= self.entities.len() {
            false
        } else {
            self.entities[entity.index()] == entity
        }
    }

    /// Spawns an entity and returns a spawned entity and if it has a reused id
    pub fn spawn(&mut self) -> (Entity, bool) {
        if let Some(mut entity) = self.id_dumpster.pop_front() {
            entity.set_gen(entity.gen() + 1);
            self.entities[entity.index()] = entity;
            (entity, true)
        } else {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let entity = Entity::from_gen_id(1, id);
            (entity, false)
        }
    }

    pub fn kill(&mut self, entity: Entity) -> EntityResult {
        if entity.index() >= self.entities.len() {
            return Err(EntityError::NotFromThisWorld(entity));
        }

        let stored_entity = self.entities[entity.index()];

        if stored_entity.gen() == 0 {
            return Err(EntityError::AlreadyDead(entity.id()));
        }

        if stored_entity != entity {
            return Err(EntityError::WrongGeneration {
                id: entity.id(),
                provided: entity.gen(),
                found: stored_entity.gen(),
            });
        }

        self.id_dumpster.push_back(stored_entity);
        self.entities[entity.index()].clear_gen();

        Ok(())
    }
}
