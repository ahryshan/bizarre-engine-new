use std::{any::type_name, marker::PhantomData, ptr::NonNull};

use super::untyped::Arena;

pub struct TypedArena<T> {
    arena: Arena,
    _marker: PhantomData<T>,
}

impl<T> TypedArena<T> {
    pub fn new(chunk_count: usize) -> Self {
        Self {
            arena: Arena::new(size_of::<T>() * chunk_count),
            _marker: PhantomData,
        }
    }

    pub fn alloc(&mut self, value: T) -> NonNull<T> {
        let ptr = self.arena.alloc();
        unsafe {
            std::ptr::write(ptr, value);
            NonNull::new(ptr)
                .unwrap_or_else(|| panic!("Failed to allocate ptr of type {}", type_name::<T>()))
        }
    }

    pub fn reset(&mut self) {
        self.arena.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::TypedArena;

    struct Person {
        name: &'static str,
        age: u32,
    }

    #[test]
    fn should_allocate_1() {
        let mut arena = TypedArena::new(10);

        let person = arena.alloc(Person {
            name: "John",
            age: 19,
        });
    }
}
