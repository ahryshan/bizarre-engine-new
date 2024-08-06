use command_buffer::CommandBuffer;

use crate::{
    component::component_batch::ComponentBatch,
    entity::{
        entity_commands::{EntityCmdBuilder, SpawnEntityCmd},
        Entity,
    },
    prelude::Resource,
    resource::resource_commands::{InsertResourceCmd, RemoveResourceCmd},
    system::{
        schedule::{self, Schedule},
        system_commands::AddSystemsCmd,
        system_config::IntoSystemConfigs,
        system_param::SystemParam,
    },
    world::{unsafe_world_cell::UnsafeWorldCell, World},
};

pub mod command_buffer;

pub trait Command {
    fn apply(self, world: &mut World);
}

pub struct Commands<'s> {
    buffer: &'s mut CommandBuffer,
}

impl<'s> Commands<'s> {
    pub fn new(buffer: &'s mut CommandBuffer) -> Self {
        Self { buffer }
    }

    pub fn spawn(&mut self, components: impl ComponentBatch) -> &mut Self {
        self.buffer.push(SpawnEntityCmd::new(components));
        self
    }

    pub fn spawn_empty(&mut self) -> &mut Self {
        self.spawn(());
        self
    }

    pub fn entity(&mut self, entity: Entity) -> EntityCmdBuilder<false> {
        EntityCmdBuilder::new(self.buffer, entity)
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.buffer.push(InsertResourceCmd::new(resource));
        self
    }

    pub fn remove_resource<T: Resource>(&mut self) -> &mut Self {
        self.buffer.push(RemoveResourceCmd::<T>::new());
        self
    }

    pub fn add_systems<M>(
        &mut self,
        schedule: Schedule,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        self.buffer
            .push(AddSystemsCmd::new(schedule, systems.into_system_configs()));
        self
    }
}

impl SystemParam for Commands<'_> {
    type Item<'w, 's> = Commands<'s>;

    type State = CommandBuffer;

    unsafe fn init(_: UnsafeWorldCell) -> Self::State {
        CommandBuffer::new()
    }

    unsafe fn get_item<'w, 's>(
        _: UnsafeWorldCell<'w>,
        state: &'s mut Self::State,
    ) -> Self::Item<'w, 's>
    where
        Self: Sized,
    {
        Commands::new(state)
    }

    fn param_access() -> Vec<crate::system::WorldAccess> {
        vec![]
    }

    fn take_deferred(state: &mut Self::State) -> Option<CommandBuffer> {
        if state.is_empty() {
            None
        } else {
            let mut buffer = CommandBuffer::new();
            buffer.append(state);
            Some(buffer)
        }
    }
}
