use std::{
    mem::MaybeUninit,
    ptr::{addr_of_mut, NonNull},
};

use crate::world::World;

use super::Command;

#[derive(Default)]
pub struct CommandBuffer {
    bytes: Vec<MaybeUninit<u8>>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    unsafe fn get_raw(&mut self) -> RawCommandBuffer {
        RawCommandBuffer {
            bytes: NonNull::new_unchecked(addr_of_mut!(self.bytes)),
        }
    }

    pub fn apply(&mut self, world: &mut World) {
        unsafe { self.get_raw().apply_or_drop_queued(Some(world.into())) }
    }

    pub fn append(&mut self, other: &mut CommandBuffer) {
        self.bytes.append(&mut other.bytes)
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.get_raw().apply_or_drop_queued(None);
        }
    }
}

pub struct CommandMeta {
    consume: unsafe fn(command: *mut (), Option<NonNull<World>>, cursor: &mut usize),
}

pub struct RawCommandBuffer {
    bytes: NonNull<Vec<MaybeUninit<u8>>>,
}

impl RawCommandBuffer {
    pub unsafe fn push<T: Command>(&mut self, cmd: T) {
        #[repr(C, packed)]
        struct Packed<T> {
            meta: CommandMeta,
            value: T,
        }

        let meta = CommandMeta {
            consume: |command, world, cursor| {
                let command: T = command.cast::<T>().read_unaligned();
                if let Some(mut world) = world {
                    let world = world.as_mut();
                    command.apply(world)
                } else {
                    drop(command)
                }
            },
        };

        let bytes = self.bytes.as_mut();
        let old_len = bytes.len();

        bytes.reserve(size_of::<Packed<T>>());

        bytes
            .as_mut_ptr()
            .add(old_len)
            .cast::<Packed<T>>()
            .write_unaligned(Packed { meta, value: cmd });

        bytes.set_len(old_len + size_of::<Packed<T>>());
    }

    pub unsafe fn apply_or_drop_queued(&mut self, world: Option<NonNull<World>>) {
        let mut cursor = 0;

        while cursor < self.bytes.as_ref().len() {
            let meta = self
                .bytes
                .as_mut()
                .as_mut_ptr()
                .cast::<CommandMeta>()
                .read();

            cursor += size_of::<CommandMeta>();

            let ptr = self.bytes.as_mut().as_mut_ptr().add(cursor).cast();

            (meta.consume)(ptr, world, &mut cursor);
        }

        self.bytes.as_mut().set_len(0);
    }
}
