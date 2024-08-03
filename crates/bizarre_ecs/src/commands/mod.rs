use command_buffer::CommandBuffer;

use crate::{
    system::system_param::SystemParam,
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

    fn take_deferred(state: &mut Self::State) -> Vec<CommandBuffer> {
        if state.is_empty() {
            vec![]
        } else {
            let mut buffer = CommandBuffer::new();
            buffer.append(state);
            vec![buffer]
        }
    }
}
