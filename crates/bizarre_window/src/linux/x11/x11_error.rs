use xcb::{x, ConnError, ProtocolError};

use crate::window_error::WindowError;

impl From<ProtocolError> for WindowError {
    fn from(value: ProtocolError) -> Self {
        use ProtocolError::*;

        #[allow(unreachable_patterns)]
        match value {
            X(_, reason) => WindowError::ProtocolError {
                protocol: "X",
                reason: reason.unwrap_or(""),
            },
            _ => WindowError::ProtocolError {
                protocol: "[unknown]",
                reason: "",
            },
        }
    }
}

impl From<xcb::Error> for WindowError {
    fn from(value: xcb::Error) -> Self {
        match value {
            xcb::Error::Connection(_) => WindowError::LostConnection,
            xcb::Error::Protocol(proto_err) => proto_err.into(),
        }
    }
}
