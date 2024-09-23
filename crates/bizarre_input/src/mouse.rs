use bizarre_input_proc_macros::define_keys;

define_keys! {
    #[derive(Clone, Copy, Debug)]
    pub MouseButton {
        (linux: 0) => Left,
        (linux: 0) => Middle,
        (linux: 0) => Right,
    }
}
