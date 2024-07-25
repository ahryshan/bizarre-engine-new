use crate::{component::Component, world::World};

use super::Entity;

pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(world: &'a mut World) -> Self {
        let entity = world.create_entity();
        Self { world, entity }
    }

    pub fn with_component<C: Component>(self, component: C) -> Self {
        self.world.register_component::<C>();
        self.world.insert_component(self.entity, component).unwrap();
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
