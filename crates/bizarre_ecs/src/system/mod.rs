use crate::{
    query::{query_data::QueryData, Query},
    world::{world_unsafe_cell::UnsafeWorldCell, World},
};

pub mod error;
pub mod schedule;
pub mod system_graph;

pub trait System {
    type QueryData<'q>: QueryData<'q> = ();

    fn init(&mut self, world: &mut World) {
        let _ = world;
    }
    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>);
    fn dispose(&mut self, world: &mut World) {
        let _ = world;
    }
}

impl<D> System for fn(Query<D>)
where
    D: for<'b> QueryData<'b>,
{
    type QueryData<'q> = D;

    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>) {
        self(query)
    }
}

trait RunFn = for<'a> Fn(*mut (), UnsafeWorldCell<'a>);
trait InitFn = for<'a> Fn(*mut (), UnsafeWorldCell<'a>);
trait DisposeFn = for<'a> Fn(*mut (), UnsafeWorldCell<'a>);

pub struct StoredSystem {
    state: *mut (),
    init_fn: Option<Box<dyn InitFn>>,
    run_fn: Box<dyn RunFn>,
    dispose_fn: Option<Box<dyn DisposeFn>>,
}

impl StoredSystem {
    pub fn init(&self, world: &World) {
        if let Some(func) = &self.init_fn {
            let cell = unsafe { UnsafeWorldCell::new(world) };
            (func)(self.state, cell)
        }
    }

    pub fn run(&self, world: &World) {
        let cell = unsafe { UnsafeWorldCell::new(world) };
        (self.run_fn)(self.state, cell)
    }

    pub fn dispose(&self, world: &mut World) {
        if let Some(func) = &self.dispose_fn {
            let cell = unsafe { UnsafeWorldCell::new(world) };
            (func)(self.state, cell)
        }
    }
}

pub trait IntoStoredSystem {
    fn into_stored_system(self) -> StoredSystem;
}

// impl<D> IntoStoredSystem for for<'a> fn(Query<'a, D>)
// where
//     D: for<'a> QueryData<'a>,
// {
//     fn into_stored_system(self) -> StoredSystem {
//         let state = { Box::into_raw(Box::new(self)) as *mut _ };
//         let run_fn = |this: *mut (), world: UnsafeWorldCell| {
//             let (this, world) = unsafe { ((this as *mut Self).as_ref().unwrap(), world.get()) };
//
//             let query = world.query();
//
//             this(query)
//         };
//
//         let run_fn = Box::new(run_fn);
//
//         StoredSystem {
//             state,
//             run_fn,
//             init_fn: None,
//             dispose_fn: None,
//         }
//     }
// }

impl<S: System> IntoStoredSystem for S {
    fn into_stored_system(self) -> StoredSystem {
        let state = {
            let boxed = Box::new(self);
            Box::into_raw(boxed) as *mut _
        };

        let init_fn = |this: *mut (), world: UnsafeWorldCell| {
            let (this, world) = unsafe { (&mut *this.cast(), world.get_mut()) };

            S::init(this, world)
        };

        let init_fn = Box::new(init_fn);

        let run_fn = |this: *mut (), world: UnsafeWorldCell| {
            let (this, world) = unsafe { (&mut *this.cast(), world.get()) };

            let query = world.query();

            S::run(this, query)
        };

        let run_fn = Box::new(run_fn);

        let dispose_fn = |this: *mut (), world: UnsafeWorldCell| {
            let (this, world) = unsafe { (&mut *this.cast(), world.get_mut()) };

            S::dispose(this, world);
        };

        let dispose_fn = Box::new(dispose_fn);

        StoredSystem {
            state,
            init_fn: Some(init_fn),
            run_fn,
            dispose_fn: Some(dispose_fn),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        query::fetch::Fetch,
        test_commons::{Health, Mana},
        world::World,
    };

    use super::{IntoStoredSystem, System};

    struct HelloWorldSystem {
        healthy_entities: usize,
    }

    impl System for HelloWorldSystem {
        type QueryData<'q> = Fetch<'q, Health>;

        fn run<'q>(&mut self, query: crate::query::Query<'q, Self::QueryData<'q>>) {
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
