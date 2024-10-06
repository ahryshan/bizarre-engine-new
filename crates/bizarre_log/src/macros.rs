#[macro_export]
macro_rules! escape_sequence {
    ($($code:expr),*) => {
        TerminalEscapeSequence{0: std::collections::HashSet::from([$($code),*])}
    };
}

#[macro_export]
macro_rules! log {
    ($name: expr, $log_level: expr, $($args:tt)*) => {
        $crate::send_log($crate::Log {
            target: $name,
            level: $log_level,
            message: format!($($args)*),
        })
    };
}

macro_rules! gen_log_macro_impl {
    ($macro_name: tt, $default_logger: tt, $log_level: tt) => {
        #[macro_export]
        macro_rules! $macro_name {
            ($$logger: tt: $$($$args: tt)*) => {
                $crate::log!(stringify!($$logger), $crate::LogLevel::$log_level, $$($$args)*)
            };
            ($$($$args: tt)*) => {
                $crate::log!(stringify!($default_logger), $crate::LogLevel::$log_level, $$($$args)*)
            }
        }

        pub(crate) use $macro_name;
    }
}

macro_rules! gen_log_macros {
    ($($logger: tt {
        $($macro_name:tt => $log_level: ident),+ $(,)?
    });+;) => {
        $($(gen_log_macro_impl!($macro_name, $logger, $log_level);)+)+
    };
}

gen_log_macros! {
    engine {
        core_trace => Trace,
        core_info => Info,
        core_warn => Warn,
        core_error => Error,
        core_fatal => Fatal,
    };
    app {
        trace => Trace,
        info => Info,
        warning => Warn,
        error => Error,
        fatal => Fatal,
    };
}
