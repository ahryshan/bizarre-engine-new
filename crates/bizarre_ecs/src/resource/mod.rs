use std::{
    any::{type_name, TypeId},
    collections::BTreeSet,
    mem::MaybeUninit,
    ptr::NonNull,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(TypeId);

pub trait Resource
where
    Self: ResourceMarker + 'static,
{
    fn id() -> ResourceId {
        ResourceId(TypeId::of::<Self>())
    }

    fn name() -> &'static str {
        type_name::<Self>()
    }
}

impl<T> Resource for T where T: ResourceMarker + 'static {}

pub auto trait ResourceMarker {}
impl !ResourceMarker for Stored {}

pub struct Stored {
    id: ResourceId,
    name: &'static str,
    data: NonNull<()>,
}

impl Stored {
    pub fn from_storable<T: IntoStored>(value: T) -> Self {
        value.into_stored()
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
    fn into_stored(self) -> Stored;
}

impl<T: Resource> IntoStored for T {
    fn into_stored(self) -> Stored {
        Stored {
            id: T::id(),
            name: T::name(),
            data: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(self)).cast()) },
        }
    }
}

impl IntoStored for Stored {
    fn into_stored(self) -> Stored {
        self
    }
}

pub struct ComponentBuffer {
    id: ResourceId,
    name: &'static str,
    item_size: usize,
    drop_fn: unsafe fn(NonNull<u8>),
    data: Vec<MaybeUninit<u8>>,
    /// Offsets in bytes to valid objects in buffer
    valid_offsets: BTreeSet<usize>,
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
            valid_offsets: Default::default(),
        }
    }

    pub fn new<T: Resource>() -> Self {
        Self::with_capacity::<T>(0)
    }

    pub fn get<T>(&self, index: usize) -> Option<&T> {
        let offset = index * self.item_size;

        if !self.valid_offsets.contains(&offset) {
            return None;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(offset) as *const T;
            Some(&*ptr)
        }
    }

    pub fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        let offset = index * self.item_size;

        if !self.valid_offsets.contains(&offset) {
            return None;
        }

        unsafe {
            let ptr = self.data.as_mut_ptr().add(offset) as *mut T;
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
    pub unsafe fn get_mut_unchecked<T>(&mut self, index: usize) -> *const T {
        self.data.as_mut_ptr().cast::<T>().add(index)
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
            let this = &mut *self;
            if this.is_valid(index) {
                let val = unsafe { this.get_unchecked::<T>(index).read() };
                Some(val)
            } else {
                None
            }
        };

        unsafe {
            let ptr: *mut T = self.data.as_mut_ptr().cast();
            ptr.write(value)
        };

        if prev_value.is_none() {
            self.valid_offsets.insert(index * self.item_size);
        }

        prev_value
    }

    pub fn remove<T>(&mut self, index: usize) -> Option<T> {
        if self.is_valid(index) {
            let val = unsafe { self.get_unchecked::<T>(index).read() };
            self.valid_offsets.remove(&(index * self.item_size));
            Some(val)
        } else {
            None
        }
    }

    pub fn is_valid(&self, index: usize) -> bool {
        let offset = index * self.item_size;
        self.valid_offsets.contains(&offset)
    }
}
