use std::fmt::Debug;

use crate::query::query_element::QueryElement;

pub mod builder;
pub mod entities;
pub mod error;

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

impl<'q> QueryElement<'q> for Entity {
    type Item = Entity;

    fn inner_type_id() -> Option<std::any::TypeId> {
        None
    }

    fn get_item(world: &'q crate::world::World, entity: Entity) -> Self::Item {
        let _ = world;
        entity
    }
}
