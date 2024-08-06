use bizarre_utils::mass_impl;

use crate::entity::Entity;

use super::{Component, ComponentRegistry};

pub trait ComponentBatch {
    fn register(components: &mut ComponentRegistry);
    fn insert(self, components: &mut ComponentRegistry, entity: Entity);
    fn remove(components: &mut ComponentRegistry, entity: Entity);
}

impl ComponentBatch for () {
    fn register(_: &mut ComponentRegistry) {}
    fn insert(self, _: &mut ComponentRegistry, _: Entity) {}
    fn remove(_: &mut ComponentRegistry, _: Entity) {}
}

impl<T: Component> ComponentBatch for T {
    fn register(components: &mut ComponentRegistry) {
        components.register::<T>();
    }

    fn insert(self, components: &mut ComponentRegistry, entity: Entity) {
        components.insert(entity, self);
    }

    fn remove(components: &mut ComponentRegistry, entity: Entity) {
        components.remove::<Self>(entity);
    }
}

macro_rules! impl_component_batch {
    ($($comp:tt),+) => {
        #[allow(non_snake_case)]
        impl<$($comp: Component),+> ComponentBatch for ($($comp,)+) {
            fn register(components: &mut ComponentRegistry) {
                $(
                    components.register::<$comp>();
                )+
            }

            fn insert(self, components: &mut ComponentRegistry, entity: Entity) {
                let ($($comp,)+) = self;
                $(
                    components.insert(entity, $comp);
                )+
            }

            fn remove(components: &mut ComponentRegistry, entity: Entity) {
                $(components.remove::<$comp>(entity);)+
            }
        }
    };
}

mass_impl!(impl_component_batch, 16, C);
