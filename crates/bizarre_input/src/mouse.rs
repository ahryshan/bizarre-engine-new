use bizarre_input_proc_macros::define_keys;

define_keys! {
    #[derive(Clone, Copy, Debug)]
    pub Mouse: u8 {
        (linux: 0) => Left,
        (linux: 1) => Right,
        (linux: 2) => Middle,
        (linux: 12) => WheelDown,
        (linux: 13) => WheelUp,
        (linux: 14) => WheelRight,
        (linux: 15) => WheelLeft,
    }
}
