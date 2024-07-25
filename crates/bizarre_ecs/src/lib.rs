#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(trait_alias)]
#![feature(marker_trait_attr)]

pub mod component;
pub mod entity;
pub mod query;
pub mod resource;
pub mod system;
pub mod world;

#[cfg(test)]
mod test_commons;

pub use component::Component;
pub use entity::Entity;
pub use resource::Resource;
pub use system::System;
pub use world::World;
