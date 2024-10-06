use std::marker::PhantomData;

use bizarre_core::builder::BuilderTypeState;
use chrono::Local;

use crate::{
    escape_code::{TerminalEscapeCode, TerminalEscapeSequence, RESET},
    escape_sequence,
    log_target::LogTarget,
    logger_builder::{LoggerBuilder, NoLabel, NoTargets},
    Log, LogLevel,
};

pub struct Logger {
    pub label: &'static str,
    pub min_level: LogLevel,
    pub targets: Box<[Box<dyn LogTarget + Send>]>,
}

impl Logger {
    pub fn log(&mut self, log: Log) {
        let Log {
            level,
            message,
            target,
            ..
        } = log;

        if level < self.min_level {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        let msg = format!(
            "{timestamp} [{}] {level_color}{level}{reset}: {message}",
            self.label,
            level_color = TerminalEscapeSequence::from(&level),
            reset = escape_sequence!(RESET)
        );

        let msg_no_color = format!("{timestamp} [{}] {level}: {message}", self.label,);

        for log_target in self.targets.iter_mut() {
            if log_target.supports_color() {
                log_target.write(msg.clone(), level, target);
            } else {
                log_target.write(msg_no_color.clone(), level, target);
            }
        }
    }

    pub fn builder() -> LoggerBuilder<NoTargets, NoLabel> {
        LoggerBuilder::<NoTargets, NoLabel>::new()
    }
}
