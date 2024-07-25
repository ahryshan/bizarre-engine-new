use thiserror::Error;

use crate::entity::Entity;

pub type ComponentResult<T = ()> = Result<T, ComponentError>;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("{0:?} does not have component `{1}`")]
    NotPresentForEntity(Entity, &'static str),

    #[error("Expected `{found}` to be `{expected}`")]
    WrongType {
        expected: &'static str,
        found: &'static str,
    },

    #[error("Component `{1}` is already present for {0:?}")]
    AlreadyPresentForEntity(Entity, &'static str),

    #[error("Trying to insert to index {index} while storage len is only {len}")]
    OutOfBounds { index: usize, len: usize },

    #[error("Component `{0}` is not present in this storage")]
    NotPresentStorage(&'static str),
}
