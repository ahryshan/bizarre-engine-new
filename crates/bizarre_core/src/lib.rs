#![feature(const_trait_impl)]
#![feature(alloc_layout_extra)]

pub mod bit_buffer;
pub mod builder;
pub mod erased_buffer;
pub mod handle;
pub mod utils;

pub use handle::{Handle, IntoHandleRawValue};
