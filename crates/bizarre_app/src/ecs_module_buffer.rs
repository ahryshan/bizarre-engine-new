use std::{
    mem::{size_of, MaybeUninit},
    ptr::NonNull,
};

use bizarre_ecs::world::{ecs_module::EcsModule, World};

#[derive(Default)]
pub struct EcsModuleBuffer {
    bytes: Vec<MaybeUninit<u8>>,
}

struct ModuleMeta {
    consume: unsafe fn(data: NonNull<()>, world: Option<NonNull<World>>, cursor: &mut usize),
}

impl EcsModuleBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_module<T: EcsModule>(&mut self, module: T) {
        #[repr(C, packed)]
        struct Packed<I> {
            meta: ModuleMeta,
            data: I,
        }

        let meta = ModuleMeta {
            consume: |data, world, cursor| {
                *cursor += size_of::<T>();
                let module = unsafe { data.cast::<T>().read_unaligned() };
                if let Some(mut world) = world {
                    module.apply(unsafe { world.as_mut() });
                } else {
                    drop(module)
                }
            },
        };

        let old_len = self.bytes.len();
        self.bytes.reserve(size_of::<Packed<T>>());

        unsafe {
            self.bytes
                .as_mut_ptr()
                .add(old_len)
                .cast::<Packed<T>>()
                .write_unaligned(Packed { meta, data: module });

            self.bytes.set_len(old_len + size_of::<Packed<T>>());
        }
    }

    pub fn apply(&mut self, world: &mut World) {
        unsafe { self.apply_or_drop(Some(world.into())) };
    }

    unsafe fn apply_or_drop(&mut self, world: Option<NonNull<World>>) {
        let mut cursor = 0;

        while cursor < self.bytes.len() {
            let meta = unsafe {
                self.bytes
                    .as_mut_ptr()
                    .add(cursor)
                    .cast::<ModuleMeta>()
                    .read_unaligned()
            };

            cursor += size_of::<ModuleMeta>();

            unsafe {
                let data = self.bytes.as_mut_ptr().add(cursor).cast::<()>();
                let data = NonNull::new_unchecked(data);
                (meta.consume)(data, world.clone(), &mut cursor)
            }
        }

        unsafe { self.bytes.set_len(0) }
    }
}

impl Drop for EcsModuleBuffer {
    fn drop(&mut self) {
        unsafe { self.apply_or_drop(None) }
    }
}
