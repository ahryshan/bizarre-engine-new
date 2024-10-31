#![feature(auto_traits)]
#![feature(fn_traits)]
#![feature(negative_impls)]
#![feature(trait_alias)]
#![feature(type_changing_struct_update)]

pub mod commands;
pub mod component;
pub mod entity;
pub mod query;
pub mod resource;
pub mod system;
pub mod world;

pub mod prelude {
    pub use crate::{
        component::{component_batch::ComponentBatch, Component, ComponentRegistry},
        entity::Entity,
        query::Query,
        resource::{Resource, ResourceId},
        system::{
            local::{FromWorld, Local},
            system_param::{Res, ResMut},
            IntoSystem, System,
        },
    };
}
