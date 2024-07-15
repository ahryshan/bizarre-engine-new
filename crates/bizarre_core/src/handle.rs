use std::{fmt::Debug, marker::PhantomData};

#[derive(Hash)]
pub struct Handle<T> {
    handle: usize,
    _marker: PhantomData<T>,
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

impl<T> Handle<T> {
    pub fn as_raw(&self) -> usize {
        self.handle
    }

    pub fn from_raw<U>(raw: U) -> Self
    where
        U: Into<Self>,
    {
        raw.into()
    }
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
    };

    ($from_ty:ty, $($width:literal),+$(,)?) => {
        #[cfg(any($(target_pointer_width = $width),+))]
        impl_from_for_handle!($from_ty);
    }
}

impl_from_for_handle!(u16, "16", "32", "64");
impl_from_for_handle!(u32, "32", "64");
impl_from_for_handle!(u64, "64");

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
