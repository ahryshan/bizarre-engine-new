use crate::world::World;

use super::{IntoSystem, System};

#[derive(Default)]
pub struct SystemGraph {
    systems: Vec<Box<dyn System>>,
}

impl SystemGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system<M, S>(&mut self, system: S)
    where
        S: IntoSystem<M> + 'static,
    {
        self.systems.push(Box::new(system.into_system()))
    }

    pub fn init_systems(&mut self, world: &World) {
        let cell = unsafe { world.as_unsafe_cell() };

        self.systems
            .iter_mut()
            .filter(|s| !s.is_init())
            .for_each(|s| s.init(cell))
    }

    pub fn run_systems(&mut self, world: &mut World) {
        let cell = unsafe { world.as_unsafe_cell() };

        self.systems
            .iter_mut()
            .filter(|s| s.is_init())
            .for_each(|system| system.run(cell));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn System>> {
        self.systems.iter()
    }
}
