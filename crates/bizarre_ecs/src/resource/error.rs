use thiserror::Error;

pub type ResourceResult<T = ()> = Result<T, ResourceError>;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Resource `{0}` is already present")]
    AlreadyPresent(&'static str),
    #[error("Resource `{0}` is not present")]
    NotPresent(&'static str),
}
