use thiserror::Error;

use super::Entity;

pub type EntityResult<T = ()> = Result<T, EntityError>;

#[derive(Error, Debug)]
pub enum EntityError {
    #[error("There is no {0:?} in this `World`")]
    NotFromThisWorld(Entity),

    #[error("Wrong generation for `Entity` with id = {id}. Provided: {provided}, found: {found}")]
    WrongGeneration { id: u64, provided: u16, found: u16 },

    #[error("Entity with id = {0} is already dead")]
    AlreadyDead(u64),
}
