use std::{marker::PhantomData, mem::MaybeUninit};

use bizarre_core::builder::BuilderTypeState;
use bizarre_ecs::{
    system::{schedule::Schedule, system_param::ResMut},
    world::{ecs_module::EcsModule, World},
};
use bizarre_event::EventQueue;
use bizarre_log::init_logging;

use crate::{
    app_event::AppEvent, default_app_module::DefaultAppEcsModule,
    ecs_module_buffer::EcsModuleBuffer, App,
};

pub struct AppBuilder<NameValidation: BuilderTypeState> {
    name: Option<String>,
    modules: EcsModuleBuffer,
    _phantom: PhantomData<NameValidation>,
}

impl<T: BuilderTypeState> AppBuilder<T> {
    pub fn new_empty() -> AppBuilder<NoName> {
        AppBuilder::<NoName> {
            name: None,
            modules: EcsModuleBuffer::default(),
            _phantom: PhantomData,
        }
    }

    pub fn with_name(self, name: impl Into<String>) -> AppBuilder<WithName> {
        AppBuilder::<WithName> {
            name: Some(name.into()),
            _phantom: PhantomData,
            ..self
        }
    }

    pub fn with_module(mut self, module: impl EcsModule) -> Self {
        self.modules.add_module(module);
        self
    }
}

impl AppBuilder<WithName> {
    /// Builds an `App`
    ///
    /// Builds an `App` and inserts all the provided [`EcsModules`][EcsModule] into the [`World`]
    /// belonging to the built `App`. Also, worth mentioning that call to `build` will initialize
    /// [`Schedule::Init`], [`Schedule::Preupdate`] and [`Schedule::Update`] and run the `Schedule::Init` once
    ///
    pub fn build(self) -> App {
        let AppBuilder {
            name, mut modules, ..
        } = self;

        init_logging(None, None);

        let name = name.expect("Cannot build an app without a name");

        let mut world = World::new();

        let mut event_queue = EventQueue::new();
        let event_reader = event_queue.create_reader();
        event_queue.register_reader::<AppEvent>(event_reader);

        world.insert_resource(event_queue);

        world.add_schedule(Schedule::Init);
        world.add_schedule(Schedule::Update);
        world.add_schedule(Schedule::Preupdate);

        world.add_systems(Schedule::Preupdate, change_event_queue_frames);

        modules.apply(&mut world);

        world.init_schedule(Schedule::Init);
        world.run_schedule(Schedule::Init);

        #[cfg(target_os = "linux")]
        let termination_receiver = setup_termination_handler();

        App {
            name,
            running: false,
            paused: false,
            world,
            event_reader,

            #[cfg(target_os = "linux")]
            termination_receiver,
        }
    }
}

impl Default for AppBuilder<NoName> {
    fn default() -> Self {
        let mut modules = EcsModuleBuffer::default();

        modules.add_module(DefaultAppEcsModule);

        Self {
            name: Default::default(),
            modules,
            _phantom: PhantomData,
        }
    }
}

pub fn change_event_queue_frames(mut eq: ResMut<EventQueue>) {
    eq.change_frames();
}

#[cfg(target_os = "linux")]
pub fn setup_termination_handler() -> std::sync::mpsc::Receiver<i32> {
    use std::sync::mpsc;

    use bizarre_log::core_info;

    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        core_info!("GOT SIGTERM");
        tx.send(0)
            .unwrap_or_else(|_| panic!("Could not send termination signal"));
    });

    rx
}

pub struct WithName;
impl BuilderTypeState for WithName {}

pub struct NoName;
impl BuilderTypeState for NoName {}
