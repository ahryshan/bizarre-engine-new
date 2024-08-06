use crate::{commands::command_buffer::CommandBuffer, world::World};

use super::system_config::{IntoSystemConfigs, SystemConfig, SystemConfigs};

pub struct SystemGraph {
    systems: Vec<SystemConfig>,
}

impl SystemGraph {
    pub fn new() -> Self {
        Self {
            systems: Default::default(),
        }
    }

    pub fn add_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) {
        let systems = systems.into_system_configs();

        match systems {
            SystemConfigs::Config(config) => self.systems.push(config),
            SystemConfigs::Configs(configs) => {
                configs.into_iter().for_each(|s| self.add_systems(s))
            }
        }
    }

    pub fn init_systems(&mut self, world: &World) {
        self.systems
            .iter_mut()
            .filter(|s| !s.system.is_init())
            .for_each(|s| s.system.init(unsafe { world.as_unsafe_cell() }))
    }

    pub fn run_systems(&mut self, world: &mut World) -> CommandBuffer {
        self.systems
            .iter_mut()
            .filter(|s| s.system.is_init())
            .filter_map(|s| {
                s.system.run(unsafe { world.as_unsafe_cell() });
                s.system.take_deferred()
            })
            .fold(CommandBuffer::new(), |mut acc, mut curr| {
                acc.append(&mut curr);
                acc
            })
    }
}

impl Default for SystemGraph {
    fn default() -> Self {
        Self::new()
    }
}
