use std::{
    any::{type_name, TypeId},
    collections::BTreeSet,
    mem::MaybeUninit,
    ptr::NonNull,
};

pub use bizarre_ecs_proc_macro::Resource;

pub mod resource_commands;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(TypeId);

pub trait Resource: 'static {
    fn resource_id() -> ResourceId {
        ResourceId(TypeId::of::<Self>())
    }

    fn resource_name() -> &'static str {
        type_name::<Self>()
    }
}

pub struct ResourceMeta {
    pub name: &'static str,
    pub id: ResourceId,
    pub size: usize,
    pub drop_fn: unsafe fn(NonNull<u8>),
}

impl ResourceMeta {
    pub fn new<T: Resource>() -> Self {
        Self {
            name: T::resource_name(),
            id: T::resource_id(),
            size: size_of::<T>(),
            drop_fn: |ptr| {
                let ptr = ptr.cast::<T>();
                let value = unsafe { ptr.read() };
                drop(value)
            },
        }
    }
}

pub struct StoredResource {
    pub(crate) id: ResourceId,
    pub(crate) name: &'static str,
    pub(crate) data: NonNull<u8>,
    pub(crate) drop_fn: unsafe fn(NonNull<u8>),
}

impl StoredResource {
    pub fn from_storable<T: IntoStored>(value: T) -> Self {
        value.into_stored()
    }

    pub unsafe fn from_meta_and_data(meta: ResourceMeta, data: NonNull<u8>) -> Self {
        let ResourceMeta {
            name, id, drop_fn, ..
        } = meta;

        Self {
            name,
            id,
            data,
            drop_fn,
        }
    }

    pub unsafe fn as_ref<T>(&self) -> &T {
        self.data.cast().as_ref()
    }

    pub unsafe fn as_mut<T>(&mut self) -> &mut T {
        self.data.cast().as_mut()
    }

    pub unsafe fn into_inner<T>(self) -> T {
        self.data.cast().read()
    }

    pub unsafe fn as_ptr_mut<T>(&mut self) -> *mut T {
        self.data.cast().as_ptr()
    }

    pub unsafe fn as_ptr<T>(&self) -> *const T {
        self.data.cast().as_ptr()
    }
}

impl Drop for StoredResource {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.data) }
    }
}

pub trait IntoStored {
    fn into_stored(self) -> StoredResource;
}

impl<T: Resource> IntoStored for T {
    fn into_stored(self) -> StoredResource {
        StoredResource {
            id: T::resource_id(),
            name: T::resource_name(),
            data: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(self)).cast()) },
            drop_fn: |ptr| {
                let ptr = ptr.cast::<T>();
                let value = unsafe { ptr.read() };
                drop(value)
            },
        }
    }
}

impl IntoStored for StoredResource {
    fn into_stored(self) -> StoredResource {
        self
    }
}
