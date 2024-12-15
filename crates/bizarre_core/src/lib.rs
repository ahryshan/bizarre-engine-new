#![feature(const_trait_impl)]

pub mod builder;
pub mod handle;
pub mod utils;

pub use handle::{Handle, IntoHandleRawValue};
