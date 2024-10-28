use std::{
    env,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::LogLevel;

pub trait LogTarget {
    /// Write to the log target
    ///
    /// `message` provided to this function is a fully formatted
    /// message that look as it must appear on the log. All other
    /// parameters are passed as metadata and should be used only for
    /// some dessision making done by the target itself
    /// (e.g. [`TerminalTarget`] will log to
    /// stderr if `level` is higher or equals to [`LogLevel::Error`])
    fn write(&mut self, message: String, level: LogLevel, target: &'static str);
    fn supports_color(&self) -> bool;
}

pub struct TerminalTarget {
    pub(crate) stderr: bool,
}

impl Default for TerminalTarget {
    fn default() -> Self {
        Self { stderr: true }
    }
}

impl LogTarget for TerminalTarget {
    fn supports_color(&self) -> bool {
        true
    }

    fn write(&mut self, message: String, level: LogLevel, _: &'static str) {
        if self.stderr && level >= LogLevel::Error {
            eprintln!("{message}")
        } else {
            println!("{message}")
        }
    }
}

pub struct FileTarget {
    pub(crate) file: PathBuf,
}

impl FileTarget {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        let path: PathBuf = file.into();

        let path = if path.is_absolute() {
            path
        } else {
            let mut prefix = env::current_exe().unwrap();
            prefix.pop();
            prefix.push(path);
            prefix
        };

        Self { file: path }
    }
}

impl LogTarget for FileTarget {
    fn supports_color(&self) -> bool {
        false
    }

    fn write(&mut self, message: String, _: LogLevel, _: &'static str) {
        if let Some(parent) = self.file.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("Failed to create log file: {err}"));
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.file)
            .unwrap_or_else(|err| panic!("Could not open file for logging: {err}"));

        file.write(&format!("{message}\n").into_bytes())
            .unwrap_or_else(|err| panic!("Could not write to file: {err}"));
    }
}