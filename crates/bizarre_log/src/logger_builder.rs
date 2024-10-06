use std::marker::PhantomData;

use crate::logger::Logger;
use crate::LogLevel;

use crate::log_target::LogTarget;

use bizarre_core::builder::BuilderTypeState;

pub struct HasTargets;

pub struct NoTargets;

pub struct HasLabel;

pub struct NoLabel;

impl BuilderTypeState for HasTargets {}

impl BuilderTypeState for NoTargets {}

impl BuilderTypeState for HasLabel {}

impl BuilderTypeState for NoLabel {}

pub struct LoggerBuilder<T: BuilderTypeState, L: BuilderTypeState> {
    pub(crate) targets: Vec<Box<dyn LogTarget + Send>>,
    pub(crate) label: Option<&'static str>,
    pub(crate) min_level: LogLevel,
    pub(crate) _phantom: PhantomData<(T, L)>,
}

impl<T: BuilderTypeState, L: BuilderTypeState> LoggerBuilder<T, L> {
    pub fn new() -> LoggerBuilder<NoTargets, NoLabel> {
        LoggerBuilder {
            targets: Vec::new(),
            label: None,
            min_level: LogLevel::default(),
            _phantom: PhantomData,
        }
    }

    pub fn with_label(self, label: &'static str) -> LoggerBuilder<T, HasLabel> {
        LoggerBuilder {
            label: Some(label),
            _phantom: PhantomData,
            ..self
        }
    }

    pub fn with_target<Target>(mut self, target: Target) -> LoggerBuilder<HasTargets, L>
    where
        Target: LogTarget + Send + 'static,
    {
        self.targets.push(Box::new(target));

        LoggerBuilder {
            _phantom: PhantomData,
            ..self
        }
    }

    pub fn with_min_level(self, level: LogLevel) -> Self {
        Self {
            min_level: level,
            ..self
        }
    }
}

impl LoggerBuilder<HasTargets, HasLabel> {
    pub fn build(self) -> Logger {
        Logger {
            label: self.label.unwrap(),
            min_level: self.min_level,
            targets: self.targets.into_boxed_slice(),
        }
    }
}
