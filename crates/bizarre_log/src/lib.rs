#![feature(type_changing_struct_update)]
#![feature(macro_metavar_expr)]

use std::fmt::Display;

mod log_thread;
pub use log_thread::{init_logging, register_logger, send_log, shutdown_logging};

pub mod escape_code;
pub mod log_target;
pub mod logger;
pub mod logger_builder;

pub mod macros;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    #[default]
    Trace,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_uppercase())
    }
}

pub struct Log {
    pub target: &'static str,
    pub level: LogLevel,
    pub message: String,
}

#[cfg(test)]
mod test {
    use crate::{
        log_target::TerminalTarget,
        log_thread::{init_logging, register_logger, shutdown_logging},
        logger::Logger,
        macros::*,
    };

    #[test]
    fn should_log() {
        init_logging(None, None);

        register_logger(
            "render",
            Logger::builder()
                .with_label("Render")
                .with_target(TerminalTarget::default())
                .build(),
        );

        trace!("Format: {}", "some string");
        info!("Format: {}", "some string");
        warning!("Format: {}", "some string");
        error!("Format: {}", "some string");
        fatal!("Format: {}", "some string");

        core_trace!("Format: {}", "some string");
        core_info!("Format: {}", "some string");
        core_warn!("Format: {}", "some string");
        core_error!("Format: {}", "some string");
        core_fatal!("Format: {}", "some string");

        shutdown_logging();
    }
}
