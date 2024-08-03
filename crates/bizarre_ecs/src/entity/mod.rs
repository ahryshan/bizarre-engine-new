use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::atomic::{self, AtomicU64},
};

use crate::query::query_element::QueryData;

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Hash, Clone, Copy)]
pub struct Entity {
    ///|gen|id |
    ///|---|---|
    ///|16b|48b|
    inner: u64,
}

impl Entity {
    const GEN_SHIFT: usize = 64 - 16;

    /// Constructs an `Entity` from generation and id.
    /// *NOTE*: an entity with generation 0 is a a special value, which is used in multiple places
    /// to note an entity that is not being considered. So the youngest valid entity is an entity
    /// with gen = 1 and id = 0
    pub const fn from_gen_id(gen: u16, id: u64) -> Self {
        let gen = (gen as u64) << Self::GEN_SHIFT;
        Self { inner: id + gen }
    }

    pub fn id(&self) -> u64 {
        self.inner << Self::GEN_SHIFT >> Self::GEN_SHIFT
    }

    pub fn index(&self) -> usize {
        self.id() as usize
    }

    pub fn gen(&self) -> u16 {
        (self.inner >> Self::GEN_SHIFT).try_into().unwrap()
    }

    pub fn set_gen(&mut self, gen: u16) {
        self.clear_gen();
        self.inner += (gen as u64) << Self::GEN_SHIFT
    }

    pub(crate) fn clear_gen(&mut self) {
        self.inner = self.inner << 16 >> 16;
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity(id: {}, gen: {})", self.id(), self.gen())
    }
}

impl QueryData for Entity {
    type Item<'w> = Self;

    fn resource_ids() -> Vec<crate::resource::ResourceId> {
        vec![]
    }

    unsafe fn get_item(
        _: crate::world::unsafe_world_cell::UnsafeWorldCell,
        entity: Entity,
    ) -> Self::Item<'_> {
        entity
    }

    fn query_access() -> Vec<crate::system::WorldAccess> {
        vec![]
    }
}

#[derive(Default)]
pub struct EntitySpawner {
    next_id: AtomicU64,
    dead: VecDeque<Entity>,
}

impl EntitySpawner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_entity(&mut self) -> (Entity, bool) {
        if let Some(mut entity) = self.dead.pop_front() {
            entity.set_gen(entity.gen() + 1);
            (entity, true)
        } else {
            let id = self.next_id.fetch_add(1, atomic::Ordering::SeqCst);
            (Entity::from_gen_id(1, id), false)
        }
    }

    pub fn kill(&mut self, entity: Entity) {
        if self.dead.contains(&entity) {
            panic!("Trying to kill an `Entity` which is already dead");
        }

        self.dead.push_back(entity)
    }
}
