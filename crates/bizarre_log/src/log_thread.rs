use std::{
    collections::BTreeMap,
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Mutex, Once, OnceLock,
    },
    thread::{self, JoinHandle},
};

use chrono::Local;

use crate::{
    log,
    log_target::{FileTarget, TerminalTarget},
    logger::Logger,
    macros::core_trace,
    Log, LogLevel,
};

static LOGGING_INIT: Once = Once::new();
static THREAD_HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

static LOG_SENDER: OnceLock<Sender<Log>> = OnceLock::new();
static LOGGER_REGISTER_SENDER: OnceLock<Sender<(&'static str, Logger)>> = OnceLock::new();

struct LogThreadContext {
    loggers: BTreeMap<&'static str, Logger>,
    log_recv: Receiver<Log>,
    register_recv: Receiver<(&'static str, Logger)>,
}

pub fn init_logging(engine_logger: Option<Logger>, app_logger: Option<Logger>) {
    LOGGING_INIT.call_once(|| {
        let (log_sender, log_recv) = channel();
        let (register_sender, register_recv) = channel();

        let log_file = format!("log/{}", Local::now().format("log_%Y_%m_%d__%H_%M_%S.log"));

        let engine_logger = match engine_logger {
            Some(logger) => logger,
            None => Logger::builder()
                .with_label("Engine")
                .with_target(TerminalTarget::default())
                .with_target(FileTarget::new(log_file.clone()))
                .build(),
        };

        let app_logger = match app_logger {
            Some(logger) => logger,
            None => Logger::builder()
                .with_label("App")
                .with_target(TerminalTarget::default())
                .with_target(FileTarget::new(log_file))
                .build(),
        };

        let mut ctx = LogThreadContext {
            loggers: BTreeMap::from([("engine", engine_logger), ("app", app_logger)]),
            log_recv,
            register_recv,
        };

        LOG_SENDER.set(log_sender);
        LOGGER_REGISTER_SENDER.set(register_sender);

        let handle = thread::spawn(move || {
            thread_body(ctx);
        });

        let _ = THREAD_HANDLE
            .lock()
            .unwrap()
            .replace(handle)
            .is_none_or(|_| panic!("Somehow logging thread got initialized more than once"));
    });
}

pub fn shutdown_logging() {
    if !LOGGING_INIT.is_completed() {
        return;
    }

    if let Some(handle) = THREAD_HANDLE.lock().unwrap().take() {
        LOG_SENDER
            .get()
            .unwrap()
            .send(Log {
                target: "__system",
                level: LogLevel::Info,
                message: "__SHUTDOWN".to_string(),
            })
            .unwrap_or_else(|err| panic!("Failed to send log: {err}"));

        handle.join().unwrap_or_else(|err| {
            panic!("Failed to join logging thread. Thread panicked: {err:?}")
        });
    }
}

pub fn send_log(log: Log) {
    LOG_SENDER
        .get()
        .unwrap()
        .send(log)
        .unwrap_or_else(|err| panic!("Could not send log: {err}"));
}

pub fn register_logger(name: &'static str, logger: Logger) {
    LOGGER_REGISTER_SENDER
        .get()
        .unwrap()
        .send((name, logger))
        .unwrap_or_else(|err| panic!("Could not register logger: {err}"));

    core_trace!("Registered logger `{name}`");
    log!(name, LogLevel::Trace, "Start of this log");
}

#[inline]
fn thread_body(mut ctx: LogThreadContext) {
    loop {
        match ctx.log_recv.recv() {
            Ok(log) => {
                if log.target == "__system" && log.message == "__SHUTDOWN" {
                    break;
                }

                match ctx.register_recv.try_recv() {
                    Ok((name, logger)) => {
                        ctx.loggers.insert(name, logger);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(err) => panic!("Failed to get log register requrest: {err}"),
                }

                if let Some(logger) = ctx.loggers.get_mut(log.target) {
                    logger.log(log);
                } else {
                    let engine_logger = ctx.loggers.get_mut("engine").unwrap();

                    engine_logger.log(Log {
                    target: "engine",
                    level: LogLevel::Error,
                    message: format!("Could not find a logger named `{}`, here's what ment to be sent to that logger:", log.target),
                });

                    engine_logger.log(Log {
                        target: "engine",
                        level: log.level,
                        message: log.message,
                    });
                }
            }
            Err(_) => todo!(),
        }
    }
}
