use super::{entities::Entities, Entity};

pub struct EntityBuilder<'a> {
    pub(crate) entities: &'a mut Entities,
    pub(crate) entity: Entity,
}

impl EntityBuilder<'_> {
    pub fn with_component<T>(self, component: T) -> Self
    where
        T: 'static,
    {
        self.entities.register_component::<T>();
        self.entities.insert_component(self.entity, component);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
