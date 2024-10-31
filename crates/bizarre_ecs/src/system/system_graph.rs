use thiserror::Error;

use crate::{commands::command_buffer::CommandBuffer, world::World};

use super::system_config::{IntoSystemConfigs, SystemConfig, SystemConfigs};

#[derive(Debug, Error)]
pub enum SystemGraphError {
    #[error("Dependencies not satisfied: {0:?}")]
    DependencyNotSatisfied(String),
}

#[derive(Clone, Debug)]
struct FailedDependencies {
    system_name: &'static str,
    before: Vec<&'static str>,
    after: Vec<&'static str>,
}

pub type SystemGraphResult<T> = Result<T, SystemGraphError>;

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
            SystemConfigs::Config(config) => self.add_system(config),
            SystemConfigs::Configs(configs) => {
                configs.into_iter().for_each(|s| self.add_systems(s))
            }
        }
    }

    pub fn init_systems(&mut self, world: &mut World) {
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

    fn check_dependencies(&self, system: SystemConfig) -> SystemGraphResult<()> {
        todo!()
    }

    fn add_system(&mut self, system: SystemConfig) {
        let after_list = system.meta.after.clone();
        let before_list = system.meta.before.clone();

        if after_list.is_empty() && before_list.is_empty() {
            self.systems.push(system);
            return;
        }

        let (names, reqs) = (
            self.systems
                .iter()
                .map(|sys| sys.meta.name)
                .enumerate()
                .rev()
                .collect::<Vec<_>>(),
            before_list,
        );

        let last_posible_pos = find_suitable_pos(names, reqs);

        let (names, reqs) = (
            self.systems
                .iter()
                .map(|sys| sys.meta.name)
                .enumerate()
                .collect::<Vec<_>>(),
            after_list,
        );

        let first_posible_pos = find_suitable_pos(names, reqs);

        if first_posible_pos <= last_posible_pos {
            self.systems.insert(last_posible_pos, system);
        } else {
            panic!(
                "Cannot insert system `{}`: it's impossible to satisfy dependencies",
                system.meta.name
            );
        }
    }
}

fn find_suitable_pos(
    names: Vec<(usize, &'static str)>,
    mut requirements: Vec<&'static str>,
) -> usize {
    let last_index = names.last().map(|val| val.0).unwrap_or(0);

    for (i, name) in names {
        requirements.retain(|req| *req != name);
        if requirements.is_empty() {
            return i;
        }
    }

    last_index
}

impl Default for SystemGraph {
    fn default() -> Self {
        Self::new()
    }
}
