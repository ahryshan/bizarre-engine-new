#[macro_export]
macro_rules! uniform_block_def {
    ($($token_tree:tt)*) => {
        #[repr(C, align(16))]
        #[derive(Clone)]
        $($token_tree)*
    };
}
