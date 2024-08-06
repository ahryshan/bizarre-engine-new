use crate::commands::Command;

use super::{schedule::Schedule, system_config::SystemConfigs};

pub struct AddSystemsCmd {
    schedule: Schedule,
    systems: SystemConfigs,
}

impl AddSystemsCmd {
    pub fn new(schedule: Schedule, systems: SystemConfigs) -> Self {
        Self { schedule, systems }
    }
}

impl Command for AddSystemsCmd {
    fn apply(self, world: &mut crate::world::World) {
        world.add_systems(self.schedule, self.systems)
    }
}
