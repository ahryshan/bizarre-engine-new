pub mod component_result;
pub mod component_storage;
pub mod entities;
pub mod entity_builder;
pub mod fetch;
pub mod fetch_mut;
pub mod query;
pub mod query_data;
pub mod query_element;
pub mod query_iterator;

use std::ops::{Add, AddAssign};

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity {
    /// [--gen(8b)--][---id(sizeof(usize)-8b)----]
    inner: usize,
}

impl Entity {
    pub const GEN_SHIFT: usize = size_of::<usize>() * 8 - 8;

    pub fn new(id: usize, gen: u8) -> Entity {
        Self {
            inner: id + ((gen as usize) << Self::GEN_SHIFT),
        }
    }

    pub fn id(&self) -> EntityId {
        EntityId(self.inner << 8 >> 8)
    }

    pub fn gen(&self) -> EntityGen {
        EntityGen(self.inner >> Self::GEN_SHIFT << Self::GEN_SHIFT)
    }
}

impl Add<EntityGen> for Entity {
    type Output = Entity;

    fn add(mut self, rhs: EntityGen) -> Self::Output {
        self.inner += rhs.0;
        self
    }
}

impl AddAssign<EntityGen> for Entity {
    fn add_assign(&mut self, rhs: EntityGen) {
        *self = *self + rhs;
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(usize);

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityGen(usize);

impl From<usize> for EntityId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<EntityId> for usize {
    fn from(value: EntityId) -> Self {
        value.0
    }
}

impl EntityId {
    pub const MAX: usize = usize::MAX << 8 >> 8;

    pub fn as_usize(&self) -> usize {
        (*self).into()
    }
}

impl From<u8> for EntityGen {
    fn from(value: u8) -> Self {
        Self((value as usize) << Entity::GEN_SHIFT)
    }
}

impl From<EntityGen> for u8 {
    fn from(value: EntityGen) -> Self {
        (value.0 >> Entity::GEN_SHIFT).try_into().unwrap()
    }
}

impl EntityGen {
    pub const MAX: u8 = u8::MAX;
}

#[cfg(test)]
mod tests {

    use crate::entity::{EntityGen, EntityId};

    use super::Entity;

    #[test]
    fn should_create_entity() {
        let entity = Entity::new(1, 0);
        assert!(entity.id().0 == 1);
        assert!(u8::from(entity.gen()) == 0);

        let entity = Entity::new(EntityId::MAX, EntityGen::MAX);
        assert!(entity.id().0 == EntityId::MAX);
        assert!(
            <EntityGen as Into<u8>>::into(entity.gen()) == EntityGen::MAX,
            "Entity's generation is not what expected, expected: {}, found: {}",
            EntityGen::MAX,
            <EntityGen as Into<u8>>::into(entity.gen())
        );
    }

    #[test]
    fn should_add_generation() {
        let entity = Entity::new(1, 0);
        let mut entity = entity + EntityGen::from(25);
        assert!(entity.gen() == EntityGen::from(25));

        entity += EntityGen::from(33);
        assert!(entity.gen() == EntityGen::from(58));
    }
}
