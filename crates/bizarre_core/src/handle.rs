use std::{
    collections::{BTreeSet, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

pub struct Handle<T: ?Sized> {
    handle: usize,
    _marker: PhantomData<T>,
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

impl<T> Handle<T> {
    pub const fn as_raw(&self) -> usize {
        self.handle
    }

    pub const fn from_raw<I: ~const IntoHandleRawValue>(raw: I) -> Self {
        Self {
            handle: raw.as_handle_raw_value(),
            _marker: PhantomData,
        }
    }

    pub const fn null() -> Self {
        Self::from_raw(usize::MAX)
    }
}

#[const_trait]
pub trait IntoHandleRawValue {
    fn as_handle_raw_value(self) -> usize;
}

macro_rules! impl_from_for_handle {
    ($from_ty:ty) => {
        impl<T> From<$from_ty> for Handle<T> {
            fn from(value: $from_ty) -> Self {
                Self {
                    handle: value as usize,
                    ..Default::default()
                }
            }
        }

        impl const IntoHandleRawValue for $from_ty {
            fn as_handle_raw_value(self) -> usize {
                self as usize
            }
        }
    };

    ($from_ty:ty, $($width:literal),+$(,)?) => {
        #[cfg(any($(target_pointer_width = $width),+))]
        impl_from_for_handle!($from_ty);
    }
}

impl_from_for_handle!(u16, "16", "32", "64");
impl_from_for_handle!(u32, "32", "64");
impl_from_for_handle!(u64, "64");
impl_from_for_handle!(usize);

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
        self._marker.hash(state);
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle.eq(&other.handle)
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.handle.cmp(&other.handle))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self {
            handle: 0,
            _marker: Default::default(),
        }
    }
}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Handle<{}>({:x})",
            std::any::type_name::<T>(),
            self.handle
        )
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            ..Default::default()
        }
    }
}

impl<T> Copy for Handle<T> {}

pub enum HandlePlacement {
    Present,
    NotPresent,
    Deleted,
    OutOfBounds,
}

pub trait HandleStrategy<T> {
    fn new_handle(&mut self, object: &T) -> (Handle<T>, bool);
    fn mark_deleted(&mut self, handle: Handle<T>);
    fn handle_placement(&self, handle: &Handle<T>) -> HandlePlacement;
}

pub trait IntoHandle {
    fn into_handle(&self) -> Handle<Self>;
}

pub struct SparseHandleStrategy<T> {
    created: BTreeSet<Handle<T>>,
    deleted: BTreeSet<Handle<T>>,
    _phantom: PhantomData<T>,
}

impl<T> Default for SparseHandleStrategy<T> {
    fn default() -> Self {
        Self {
            created: Default::default(),
            deleted: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T> SparseHandleStrategy<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: IntoHandle> HandleStrategy<T> for SparseHandleStrategy<T> {
    fn new_handle(&mut self, object: &T) -> (Handle<T>, bool) {
        let handle = object.into_handle();

        if self.deleted.contains(&handle) {
            self.deleted.remove(&handle);
        }

        self.created.insert(handle);
        (handle, false)
    }

    fn mark_deleted(&mut self, handle: Handle<T>) {
        self.created.remove(&handle);
        self.deleted.insert(handle);
    }

    fn handle_placement(&self, handle: &Handle<T>) -> HandlePlacement {
        if self.deleted.contains(&handle) {
            HandlePlacement::Deleted
        } else if self.created.contains(&handle) {
            HandlePlacement::Present
        } else {
            HandlePlacement::NotPresent
        }
    }
}

pub struct DenseHandleStrategy<T> {
    next_id: AtomicUsize,
    id_dumpster: Arc<Mutex<VecDeque<Handle<T>>>>,
    _phantom: PhantomData<T>,
}

impl<T> DenseHandleStrategy<T> {
    pub fn new() -> Self {
        Self {
            next_id: AtomicUsize::new(0),
            id_dumpster: Arc::new(Mutex::new(VecDeque::new())),
            _phantom: Default::default(),
        }
    }
}

impl<T> Default for DenseHandleStrategy<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> HandleStrategy<T> for DenseHandleStrategy<T> {
    fn new_handle(&mut self, _: &T) -> (Handle<T>, bool) {
        let mut dumpster = self
            .id_dumpster
            .lock()
            .expect("Could not lock recycled ids for addition");

        if dumpster.is_empty() {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            (Handle::from_raw(id), false)
        } else {
            (dumpster.pop_front().unwrap(), true)
        }
    }

    fn mark_deleted(&mut self, handle: Handle<T>) {
        let mut a = self
            .id_dumpster
            .lock()
            .expect("Could not lock recycled ids for addition");

        a.push_back(handle)
    }

    fn handle_placement(&self, handle: &Handle<T>) -> HandlePlacement {
        if handle.as_raw() == 0 || handle.as_raw() >= self.next_id.load(Ordering::SeqCst) {
            HandlePlacement::OutOfBounds
        } else if self
            .id_dumpster
            .lock()
            .expect("Could not lock recycled handle ids")
            .contains(handle)
        {
            HandlePlacement::Deleted
        } else {
            HandlePlacement::Present
        }
    }
}
