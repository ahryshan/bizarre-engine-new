use std::any::TypeId;

use super::component_storage::Component;

pub trait QueryElement {
    type Item: 'static;
    type LockType<'a>
    where
        Self: 'a;
    type RefType<'a>
    where
        Self: 'a;

    fn inner_type_id() -> TypeId {
        TypeId::of::<Self::Item>()
    }

    fn from_component(component: Component) -> Self;
    fn get_lock(&self) -> Self::LockType<'_>;
}
