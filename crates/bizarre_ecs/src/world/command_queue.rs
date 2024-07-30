//! Most of this implementation is borrowed from bevy_ecs
//!

use std::ptr::{addr_of_mut, NonNull};

use crate::World;

pub trait Command {
    fn apply(self, world: &mut World);
}

struct CommandMeta {
    consume: unsafe fn(cmd: NonNull<u8>, world: Option<NonNull<World>>, cursor: &mut usize),
}

#[derive(Default)]
pub struct CommandQueue {
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct RawCommandQueue {
    pub(crate) bytes: NonNull<Vec<u8>>,
}

impl Default for RawCommandQueue {
    fn default() -> Self {
        unsafe {
            Self {
                bytes: NonNull::new_unchecked(Box::into_raw(Box::default())),
            }
        }
    }
}

impl CommandQueue {
    pub fn push<C: Command>(&mut self, command: C) {
        unsafe { self.get_raw().push(command) }
    }

    pub fn apply(&mut self, world: &mut World) {
        unsafe { self.get_raw().apply_or_drop(Some(world.into())) }
    }

    pub fn append(&mut self, other: &mut Self) {
        self.bytes.append(&mut other.bytes)
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub(crate) fn get_raw(&mut self) -> RawCommandQueue {
        unsafe {
            RawCommandQueue {
                bytes: NonNull::new_unchecked(addr_of_mut!(self.bytes)),
            }
        }
    }
}

impl Drop for CommandQueue {
    fn drop(&mut self) {
        unsafe {
            self.get_raw().apply_or_drop(None);
        }
    }
}

impl RawCommandQueue {
    pub unsafe fn push<C: Command>(&mut self, command: C) {
        #[repr(C)]
        struct Packed<T> {
            meta: CommandMeta,
            command: T,
        }

        let meta = CommandMeta {
            consume: |cmd, world, cursor| {
                *cursor += size_of::<C>();
                let c: C = cmd.cast().read_unaligned();

                match world {
                    Some(mut world) => c.apply(world.as_mut()),
                    None => drop(c),
                }
            },
        };

        let bytes = self.bytes.as_mut();
        let old_len = bytes.len();
        bytes.reserve(size_of::<Packed<C>>());

        let ptr = bytes.as_mut_ptr().add(old_len);

        ptr.cast::<Packed<C>>()
            .write_unaligned(Packed { meta, command });

        bytes.set_len(old_len + size_of::<Packed<C>>());
    }

    pub unsafe fn apply_or_drop(&mut self, world: Option<NonNull<World>>) {
        let mut cursor = 0;
        let stop = self.bytes.as_ref().len();

        while cursor < stop {
            let meta = self
                .bytes
                .as_mut()
                .as_mut_ptr()
                .add(cursor)
                .cast::<CommandMeta>()
                .read_unaligned();

            cursor += size_of::<CommandMeta>();

            let cmd = NonNull::new_unchecked(self.bytes.as_mut().as_mut_ptr().add(cursor));

            (meta.consume)(cmd, world, &mut cursor);
        }

        self.bytes.as_mut().set_len(0);
    }

    pub unsafe fn append(&mut self, other: &mut Self) {
        self.bytes.as_mut().append(other.bytes.as_mut())
    }

    pub unsafe fn is_empty(&self) -> bool {
        self.bytes.as_ref().is_empty()
    }
}
