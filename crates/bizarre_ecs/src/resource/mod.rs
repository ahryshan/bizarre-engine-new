use std::{
    any::{type_name, TypeId},
    collections::BTreeSet,
    mem::MaybeUninit,
    ptr::NonNull,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(TypeId);

pub trait Resource: 'static {
    fn id() -> ResourceId {
        ResourceId(TypeId::of::<Self>())
    }

    fn name() -> &'static str {
        type_name::<Self>()
    }
}

pub struct ResourceMeta {
    pub name: &'static str,
    pub id: ResourceId,
    pub size: usize,
}

impl ResourceMeta {
    pub fn new<T: Resource>() -> Self {
        Self {
            name: T::name(),
            id: T::id(),
            size: size_of::<T>(),
        }
    }
}

pub struct StoredResource {
    pub(crate) id: ResourceId,
    pub(crate) name: &'static str,
    pub(crate) data: NonNull<u8>,
}

impl StoredResource {
    pub fn from_storable<T: IntoStored>(value: T) -> Self {
        value.into_stored()
    }

    pub unsafe fn from_meta_and_data(meta: ResourceMeta, data: NonNull<u8>) -> Self {
        let ResourceMeta { name, id, .. } = meta;

        Self { name, id, data }
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

pub trait IntoStored {
    fn into_stored(self) -> StoredResource;
}

impl<T: Resource> IntoStored for T {
    fn into_stored(self) -> StoredResource {
        StoredResource {
            id: T::id(),
            name: T::name(),
            data: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(self)).cast()) },
        }
    }
}

impl IntoStored for StoredResource {
    fn into_stored(self) -> StoredResource {
        self
    }
}

pub struct ComponentBuffer {
    id: ResourceId,
    name: &'static str,
    item_size: usize,
    drop_fn: unsafe fn(NonNull<u8>),
    data: Vec<MaybeUninit<u8>>,
    valid_indices: BTreeSet<usize>,
}

impl ComponentBuffer {
    pub fn with_capacity<T: Resource>(capacity: usize) -> Self {
        Self {
            id: T::id(),
            name: T::name(),
            item_size: size_of::<T>(),
            drop_fn: |item| {
                let item: T = unsafe { item.cast().read() };
                drop(item)
            },
            data: Vec::with_capacity(size_of::<T>() * capacity),
            valid_indices: Default::default(),
        }
    }

    pub fn new<T: Resource>() -> Self {
        Self::with_capacity::<T>(0)
    }

    pub fn get<T>(&self, index: usize) -> Option<&T> {
        if !self.valid_indices.contains(&index) {
            return None;
        }

        unsafe {
            let ptr = self.data.as_ptr().cast::<T>().add(index);
            Some(&*ptr)
        }
    }

    pub fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        if !self.valid_indices.contains(&index) {
            return None;
        }

        unsafe {
            let ptr = self.data.as_mut_ptr().cast::<T>().add(index);
            Some(&mut *ptr)
        }
    }

    /// Returns pointer to the data at the provided index
    ///
    /// # Safety
    ///
    /// Caller must ensure that this buffer has appropriate len and it has initialized value on the
    /// provided index
    pub unsafe fn get_unchecked<T>(&self, index: usize) -> *const T {
        self.data.as_ptr().cast::<T>().add(index)
    }

    /// Returns pointer to the data at the provided index
    ///
    /// # Safety
    ///
    /// Caller must ensure that this buffer has appropriate len and it has initialized value on the
    /// provided index
    pub unsafe fn get_mut_unchecked<T>(&mut self, index: usize) -> *mut T {
        self.data.as_mut_ptr().cast::<T>().add(index)
    }

    pub unsafe fn get_raw_unchecked(&mut self, index: usize) -> *mut u8 {
        self.data.as_mut_ptr().add(index * self.item_size).cast()
    }

    pub fn expand_by(&mut self, count: usize) {
        self.data.reserve(self.item_size * count);
        unsafe { self.data.set_len(self.data.len() + count * self.item_size) }
    }

    pub fn expand(&mut self) {
        self.expand_by(1)
    }

    pub fn insert<T>(&mut self, index: usize, value: T) -> Option<T> {
        let prev_value = {
            if self.is_valid(index) {
                let val = unsafe { self.get_unchecked::<T>(index).read() };
                Some(val)
            } else {
                None
            }
        };

        unsafe {
            let ptr: *mut T = self.data.as_mut_ptr().cast::<T>().add(index);
            ptr.write(value)
        };

        if prev_value.is_none() {
            self.valid_indices.insert(index);
        }

        prev_value
    }

    pub unsafe fn insert_raw(&mut self, index: usize, ptr: NonNull<u8>, size: usize) {
        if self.item_size != size {
            panic!("Trying to insert into `ComponentBuffer` an item of size {size}, when the buffer has item_size = {}", self.item_size);
        }

        let buffer_ptr = NonNull::new_unchecked(self.get_raw_unchecked(index));

        if self.is_valid(index) {
            (self.drop_fn)(buffer_ptr);
        } else {
            self.valid_indices.insert(index);
        }

        std::ptr::copy_nonoverlapping(ptr.as_ptr(), buffer_ptr.as_ptr(), size);
    }

    pub fn remove<T>(&mut self, index: usize) -> Option<T> {
        if self.is_valid(index) {
            let val = unsafe { self.get_unchecked::<T>(index).read() };
            self.valid_indices.remove(&index);
            Some(val)
        } else {
            None
        }
    }

    pub fn is_valid(&self, index: usize) -> bool {
        self.valid_indices.contains(&index)
    }
}
