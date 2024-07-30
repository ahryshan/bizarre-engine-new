use command_queue::{Command, CommandQueue, RawCommandQueue};

use crate::{
    component::{
        component_storage::IntoStoredComponent, error::ComponentResult, Component, Components,
    },
    entity::{builder::EntityBuilder, entities::Entities, error::EntityResult, Entity},
    query::{query_data::QueryData, Query},
    resource::{error::ResourceResult, Resource, Resources},
    system::{
        error::SystemResult,
        schedule::{Schedule, Schedules},
        IntoStoredSystem,
    },
};

pub mod command_queue;
pub mod commands;
pub mod world_unsafe_cell;

#[derive(Default)]
pub struct World {
    pub(crate) entities: Entities,
    pub(crate) components: Components,
    pub(crate) resources: Resources,
    pub(crate) schedules: Schedules,
    pub(crate) cmd_queue: RawCommandQueue,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    pub fn kill(&mut self, entity: Entity) -> EntityResult {
        self.entities.kill(entity)?;
        self.components.remove_entity(entity);
        Ok(())
    }

    pub fn insert_component<C: IntoStoredComponent>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> ComponentResult {
        self.components.insert(entity, component)
    }

    pub fn register_component<C: Component>(&mut self) {
        self.components.register::<C>()
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.components.remove(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        let (entity, reused) = self.entities.spawn();

        if !reused {
            self.components.expand();
        }

        entity
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> ResourceResult {
        self.resources.insert(resource)
    }

    pub fn get_resource<R: Resource>(&self) -> ResourceResult<&R> {
        self.resources.get()
    }

    pub fn get_resource_mut<R: Resource>(&mut self) -> ResourceResult<&mut R> {
        self.resources.get_mut()
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources.remove()
    }

    pub fn query<'q, D: QueryData>(&'q self) -> Query<'q, D> {
        Query::new(self)
    }

    pub fn add_system(
        &mut self,
        schedule: Schedule,
        system: impl IntoStoredSystem,
        name: &'static str,
    ) -> SystemResult {
        self.add_system_with_dependencies(schedule, system, name, &[])
    }

    pub fn add_system_with_dependencies(
        &mut self,
        schedule: Schedule,
        system: impl IntoStoredSystem,
        name: &'static str,
        dependencies: &[&'static str],
    ) -> SystemResult {
        self.schedules
            .add_system(schedule, name, dependencies, system)
    }

    pub fn frame(&mut self) -> SystemResult {
        self.run_schedule(Schedule::Frame)
    }

    pub fn tick(&mut self) -> SystemResult {
        self.run_schedule(Schedule::Tick)
    }

    pub fn init(&mut self) -> SystemResult {
        self.run_schedule(Schedule::Init)
    }

    pub fn run_schedule(&mut self, schedule: Schedule) -> SystemResult {
        let cmd = self.schedules.run(schedule, self)?;
        self.push_command_queue(cmd);
        self.flush();

        Ok(())
    }

    pub fn push_command(&mut self, cmd: impl Command) {
        unsafe {
            self.cmd_queue.push(cmd);
        }
    }

    pub fn push_command_queue(&mut self, mut queue: CommandQueue) {
        unsafe { self.cmd_queue.append(&mut queue.get_raw()) }
    }

    pub fn flush(&mut self) {
        unsafe { self.cmd_queue.clone().apply_or_drop(Some(self.into())) }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unsafe { self.cmd_queue.apply_or_drop(None) };
        drop(unsafe { Box::from_raw(self.cmd_queue.bytes.as_ptr()) });
    }
}

#[cfg(test)]
mod test {
    use crate::{
        query::{
            fetch::{Fetch, FetchMut},
            res::{Res, ResMut},
            Query,
        },
        resource::Resource,
        test_commons::{Health, Mana, Motd},
    };

    use super::World;

    #[test]
    fn should_query_for_one_component() {
        let mut world = World::new();

        world.spawn().with_component(Health(100)).build();
        world.spawn().with_component(Health(200)).build();
        world.spawn().with_component(Health(300)).build();
        world.spawn().with_component(Health(400)).build();
        world.spawn().with_component(Health(500)).build();
        world.spawn().with_component(Health(600)).build();
        world.spawn().with_component(Health(700)).build();
        world.spawn().with_component(Health(800)).build();

        let query: Query<Fetch<Health>> = world.query::<Fetch<Health>>();
        // TODO: Make it see the type!
        // let query = world.query::<Fetch<Health>>();

        for health in query {
            eprintln!("{health:?}")
        }
    }

    #[test]
    fn should_query_for_two_components() {
        let mut world = World::new();

        world
            .spawn()
            .with_component(Health(100))
            .with_component(Mana(300))
            .build();
        world
            .spawn()
            .with_component(Health(200))
            .with_component(Mana(400))
            .build();
        world
            .spawn()
            .with_component(Health(300))
            .with_component(Mana(500))
            .build();
        world
            .spawn()
            .with_component(Health(400))
            .with_component(Mana(600))
            .build();

        let query: Query<(Fetch<Health>, Fetch<Mana>)> =
            world.query::<(Fetch<Health>, Fetch<Mana>)>();

        for (health, mana) in query {
            eprintln!("({health:?}, {mana:?})");
        }
    }

    #[test]
    fn should_query_for_resource() {
        let mut world = World::new();

        world.spawn().with_component(Health(100)).build();
        world.spawn().with_component(Health(400)).build();
        world.spawn().with_component(Health(300)).build();
        world.spawn().with_component(Health(200)).build();

        world.insert_resource(Motd("Hello, World!")).unwrap();

        let query: Query<(Fetch<Health>, Res<Motd>)> = world.query();

        for (health, motd) in query {
            eprintln!("({health:?}, {motd:?})");
        }
    }

    #[test]
    fn should_query_for_component_mut() {
        let mut world = World::new();

        world.spawn().with_component(Health(100)).build();
        world.spawn().with_component(Health(200)).build();
        world.spawn().with_component(Health(300)).build();

        let query: Query<FetchMut<Health>> = world.query();

        for health in query {
            health.0 *= 2;
        }

        let query: Query<Fetch<Health>> = world.query();

        let components = query.into_iter().collect::<Vec<_>>();

        if let [&Health(200), &Health(400), &Health(600)] = components.as_slice() {
            eprintln!("{components:?}");
        } else {
            panic!()
        }
    }

    #[test]
    fn should_query_for_nonmut_and_mut_components() {
        let mut world = World::new();

        world
            .spawn()
            .with_component(Health(100))
            .with_component(Mana(1000))
            .build();
        world
            .spawn()
            .with_component(Health(200))
            .with_component(Mana(1200))
            .build();
        world
            .spawn()
            .with_component(Health(300))
            .with_component(Mana(99))
            .build();

        let query: Query<(FetchMut<Health>, Fetch<Mana>)> = world.query();

        for (health, mana) in query {
            if mana.0 >= 1000 {
                health.0 *= 2;
            }
        }

        let query: Query<Fetch<Health>> = world.query();

        let components = query.into_iter().collect::<Vec<_>>();

        if let [&Health(200), &Health(400), &Health(300)] = components.as_slice() {
            eprintln!("{components:?}");
        } else {
            panic!()
        }
    }

    #[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
    struct HealthSum(usize);

    impl Resource for HealthSum {}

    #[test]
    fn should_query_for_components_and_mut_resources() {
        let mut world = World::new();

        world.spawn().with_component(Health(100)).build();
        world.spawn().with_component(Health(200)).build();
        world.spawn().with_component(Health(300)).build();

        world.insert_resource(HealthSum(0)).unwrap();

        let query: Query<(Fetch<Health>, ResMut<HealthSum>)> = world.query();

        for (health, sum) in query {
            sum.0 += health.0;
        }

        let sum = world.get_resource::<HealthSum>().unwrap();

        assert!(sum.0 == 600);
    }
}
