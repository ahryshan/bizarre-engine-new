use std::ptr::NonNull;

use crate::{
    query::{query_data::QueryData, Query},
    world::{command_queue::CommandQueue, commands::Commands, World},
};

pub mod error;
pub mod schedule;
pub mod system_graph;

pub trait System {
    type InitData<'q>: QueryData<'q> = ();
    type RunData<'q>: QueryData<'q> = ();
    type DisposeData<'q>: QueryData<'q> = ();

    fn init<'q>(&mut self, query: Query<'q, Self::InitData<'q>>, commands: &mut Commands) {
        let _ = commands;
        let _ = query;
    }
    fn run<'q>(&mut self, query: Query<'q, Self::RunData<'q>>, commands: &mut Commands);
    fn dispose<'q>(&mut self, query: Query<'q, Self::DisposeData<'q>>, commands: &mut Commands) {
        let _ = commands;
        let _ = query;
    }
}

type InitFn = unsafe fn(NonNull<()>, NonNull<World>) -> CommandQueue;
type RunFn = unsafe fn(NonNull<()>, NonNull<World>) -> CommandQueue;
type DisposeFn = unsafe fn(NonNull<()>, NonNull<World>) -> CommandQueue;
type DropFn = unsafe fn(NonNull<()>);

pub struct StoredSystem {
    state: NonNull<()>,
    init_fn: InitFn,
    run_fn: RunFn,
    dispose_fn: DisposeFn,
    drop_fn: DropFn,
}

impl StoredSystem {
    pub fn init(&mut self, world: &World) -> CommandQueue {
        unsafe { (self.init_fn)(self.state, world.into()) }
    }

    pub fn run(&mut self, world: &World) -> CommandQueue {
        unsafe { (self.run_fn)(self.state, world.into()) }
    }

    pub fn dispose(&mut self, world: &World) -> CommandQueue {
        unsafe { (self.dispose_fn)(self.state, world.into()) }
    }
}

impl Drop for StoredSystem {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.state) }
    }
}

pub trait IntoStoredSystem {
    fn into_stored_system(self) -> StoredSystem;
}

impl<T> IntoStoredSystem for T
where
    T: System,
{
    fn into_stored_system(self) -> StoredSystem {
        StoredSystem {
            state: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(self)).cast()) },
            init_fn: |state, mut world| {
                let (state, world) = unsafe { (state.cast().as_mut(), world.as_mut()) };
                let query = world.query();
                let mut commands = Commands::default();

                Self::init(state, query, &mut commands);

                commands.into_queue()
            },
            run_fn: |state, mut world| {
                let (state, world) = unsafe { (state.cast().as_mut(), world.as_mut()) };
                let query = world.query();
                let mut commands = Commands::default();

                Self::run(state, query, &mut commands);

                commands.into_queue()
            },
            dispose_fn: |state, mut world| {
                let (state, world) = unsafe { (state.cast().as_mut(), world.as_mut()) };
                let query = world.query();
                let mut commands = Commands::default();

                Self::dispose(state, query, &mut commands);

                commands.into_queue()
            },

            drop_fn: |state| {
                let state: Self = unsafe { state.cast().read_unaligned() };
                drop(state)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        query::{fetch::Fetch, Query},
        test_commons::{Health, Mana},
        world::{commands::Commands, World},
    };

    use super::{IntoStoredSystem, System};

    struct HelloWorldSystem {
        healthy_entities: usize,
    }

    impl System for HelloWorldSystem {
        type RunData<'q> = Fetch<'q, Health>;

        fn run<'q>(&mut self, query: Query<'q, Self::RunData<'q>>, _: &mut Commands) {
            let count = query.into_iter().filter(|h| h.0 > 50).count();

            self.healthy_entities += count;

            println!(
                "Hello world! Today we've met {count} healthy entities! ({} overall)",
                self.healthy_entities
            );
        }
    }

    #[test]
    fn should_store_and_run_system() {
        let mut world = World::new();

        world.spawn().with_component(Health(100)).build();
        world.spawn().with_component(Health(50)).build();
        world.spawn().with_component(Health(200)).build();
        world.spawn().with_component(Mana(100)).build();

        let stored = HelloWorldSystem {
            healthy_entities: 0,
        }
        .into_stored_system();

        stored.run(&world);
        stored.run(&world);
        stored.run(&world);
        stored.run(&world);
        stored.run(&world);

        unsafe {
            let cast = &*stored.state.cast::<HelloWorldSystem>();
            assert!(cast.healthy_entities == 10)
        }
    }
}
