use thiserror::Error;

use super::schedule::Schedule;

pub type SystemResult<T = ()> = Result<T, SystemError>;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Could not find dependencies: `{not_found}` for system `{system_name}`")]
    NoDependency {
        system_name: &'static str,
        not_found: String,
    },
    #[error("Could not find schedule `{schedule:?}` to insert system")]
    NoSchedule { schedule: Schedule },
}
