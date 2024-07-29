use std::collections::HashMap;

use crate::World;

use super::{
    error::{SystemError, SystemResult},
    system_graph::SystemGraph,
    IntoStoredSystem,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Schedule {
    Frame,
    Tick,
    Init,
}

pub struct Schedules {
    schedules: HashMap<Schedule, SystemGraph>,
}

impl Schedules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system(
        &mut self,
        schedule: Schedule,
        name: &'static str,
        deps: &[&'static str],
        system: impl IntoStoredSystem,
    ) -> SystemResult {
        if let Some(sch) = self.schedules.get_mut(&schedule) {
            sch.add_system(system.into_stored_system(), name, deps)
        } else {
            Err(SystemError::NoSchedule { schedule })
        }
    }

    pub fn run(&self, schedule: Schedule, world: &World) -> SystemResult {
        if let Some(sch) = self.schedules.get(&schedule) {
            sch.init_systems(world);
            sch.run_systems(world);
            Ok(())
        } else {
            Err(SystemError::NoSchedule { schedule })
        }
    }
}

impl Default for Schedules {
    fn default() -> Self {
        Self {
            schedules: [
                (Schedule::Frame, SystemGraph::new()),
                (Schedule::Tick, SystemGraph::new()),
                (Schedule::Init, SystemGraph::new()),
            ]
            .into_iter()
            .collect(),
        }
    }
}
