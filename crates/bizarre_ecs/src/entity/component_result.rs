use std::any::type_name;

use thiserror::Error;

pub type ComponentResult<T> = Result<T, ComponentError>;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error(r#"Component "{component_name}" is not registered in this entity storage"#)]
    NotRegistered { component_name: &'static str },
}

impl ComponentError {
    pub fn not_registered<T>() -> ComponentError {
        ComponentError::NotRegistered {
            component_name: type_name::<T>(),
        }
    }
}
