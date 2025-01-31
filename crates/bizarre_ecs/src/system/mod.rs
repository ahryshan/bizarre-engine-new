use std::fmt::Display;

use bitflags::bitflags;
use system_param::SystemParam;

use crate::{
    commands::command_buffer::CommandBuffer,
    prelude::ResourceId,
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

pub mod functional_system;
pub mod local;
pub mod schedule;
pub mod system_commands;
pub mod system_config;
pub mod system_graph;
pub mod system_param;

bitflags! {
    #[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord)]
    pub struct WorldAccessType: u8 {
        const CompRead = Self::Comp.bits() | Self::Read.bits();
        const CompWrite = Self::Comp.bits() | Self::Write.bits();
        const ResRead = Self::Res.bits() | Self::Read.bits();
        const ResWrite = Self::Res.bits() | Self::Write.bits();

        const Comp  = 0b0100;
        const Res   = 0b1000;

        const Read  = 0b0001;
        const Write = 0b0010;

        const ResourceMask  = 0b1100;
        const RwMask        = 0b0011;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldAccess {
    pub resource_id: ResourceId,
    pub resource_name: &'static str,
    pub access_type: WorldAccessType,
}

impl Display for WorldAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read_write = match self.access_type & WorldAccessType::RwMask {
            WorldAccessType::Read => "immutable",
            WorldAccessType::Write => "mutable",
            _ => "[unknown]",
        };
        let resource_type = match self.access_type & WorldAccessType::ResourceMask {
            WorldAccessType::Res => "resource",
            WorldAccessType::Comp => "component",
            _ => "[unknown]",
        };

        write!(
            f,
            "{read_write} access to {resource_type} `{}`",
            self.resource_name
        )
    }
}

pub trait System {
    fn run(&mut self, world: UnsafeWorldCell);

    fn init(&mut self, world: UnsafeWorldCell);

    fn is_init(&self) -> bool;

    fn apply_deferred(&mut self, world: &mut World);

    fn take_deferred(&mut self) -> Option<CommandBuffer>;

    fn access() -> Box<[WorldAccess]>
    where
        Self: Sized;
}

pub trait IntoSystem<Marker> {
    type System: System + 'static;

    fn into_system(self) -> Self::System;
}
