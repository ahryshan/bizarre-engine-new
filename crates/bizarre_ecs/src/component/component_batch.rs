use std::ptr::NonNull;

use bizarre_utils::mass_impl;

use crate::{prelude::Resource, resource::ResourceMeta};

#[derive(Debug)]
pub struct ResourceBatch {
    data: Vec<u8>,
}

#[repr(C, packed)]
struct Packed<T> {
    meta: ResourceMeta,
    value: T,
}

pub struct BatchedResource(pub ResourceMeta, pub NonNull<u8>);

pub trait IntoResourceBatch {
    unsafe fn into_resource_batch(self) -> ResourceBatch;
}

impl IntoResourceBatch for () {
    unsafe fn into_resource_batch(self) -> ResourceBatch {
        ResourceBatch { data: vec![] }
    }
}

impl<T> IntoResourceBatch for T
where
    T: Resource,
{
    unsafe fn into_resource_batch(self) -> ResourceBatch {
        let packed = Packed {
            meta: ResourceMeta::new::<T>(),
            value: self,
        };

        let mut data = vec![0; size_of::<Packed<T>>()];

        data.as_mut_ptr().cast::<Packed<T>>().write(packed);

        ResourceBatch { data }
    }
}

impl IntoIterator for ResourceBatch {
    type Item = BatchedResource;

    type IntoIter = ResourceBatchIterator;

    fn into_iter(self) -> Self::IntoIter {
        ResourceBatchIterator {
            offset: 0,
            data: self.data,
        }
    }
}

pub struct ResourceBatchIterator {
    data: Vec<u8>,
    offset: usize,
}

impl Iterator for ResourceBatchIterator {
    type Item = BatchedResource;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            None
        } else {
            let meta = unsafe {
                self.data
                    .as_mut_ptr()
                    .add(self.offset)
                    .cast::<ResourceMeta>()
                    .read()
            };
            self.offset += size_of_val(&meta);

            let data =
                unsafe { NonNull::new_unchecked(self.data.as_mut_ptr().add(self.offset)).cast() };

            self.offset += meta.size;

            Some(BatchedResource(meta, data))
        }
    }
}

macro_rules! impl_into_resource_batch {
    ($($res:tt),+) => {
        impl<$($res: Resource),+> IntoResourceBatch for ($($res,)+) {
            #[allow(non_snake_case, unused)]
            unsafe fn into_resource_batch(self) -> ResourceBatch {
                let size = $(
                    size_of::<$res>() + size_of::<ResourceMeta>() +
                )+ 0;

                let ($($res,)+) = self;

                let mut data = vec![0; size];

                let ptr: *mut u8 = data.as_mut_ptr();

                $(
                    let ptr = ptr.cast::<Packed<$res>>();
                    ptr.write(Packed {
                        meta: ResourceMeta::new::<$res>(),
                        value: $res,
                    });
                    let ptr = ptr.add(1);
                )+

                ResourceBatch { data }
            }
        }
    };
}

mass_impl!(impl_into_resource_batch, 16, R);
