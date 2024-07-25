use thiserror::Error;

pub type SystemResult<T = ()> = Result<T, SystemError>;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Could not find dependencies: `{not_found}` for system `{system_name}`")]
    NoDependency {
        system_name: Box<str>,
        not_found: String,
    },
}
