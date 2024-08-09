use bizarre_ecs::{
    system::{schedule::Schedule, system_param::ResMut},
    world::{ecs_module::EcsModule, World},
};
use bizarre_event::EventQueue;

use crate::{
    window_manager::{self, WindowManager},
    PlatformWindow, WindowCreateInfo,
};
