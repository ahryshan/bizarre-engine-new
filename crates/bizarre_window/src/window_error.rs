use thiserror::Error;

pub type WindowResult<T> = Result<T, WindowError>;

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("The provided `WindowHandle` is invalid")]
    InvalidHandle,
    #[error("The underlying display server disconnected")]
    LostConnection,
    #[error("There was a problem with protocol `{protocol}`: {reason}")]
    ProtocolError {
        protocol: &'static str,
        reason: &'static str,
    },
    #[error("Error unknown")]
    Unknown,
}
