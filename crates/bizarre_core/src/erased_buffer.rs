use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

use crate::bit_buffer::BitBuffer;

pub struct ErasedSparseArray {
    element_layout: Layout,
    element_stride: usize,
    valid_elements: BitBuffer,
    capacity: usize,
    drop_fn: unsafe fn(*mut ()),
    data: *mut u8,
}

const INITIAL_CAPACITY: usize = 128;

impl ErasedSparseArray {
    pub fn new<T: Sized>() -> Self {
        Self::with_capacity::<T>(INITIAL_CAPACITY)
    }

    pub fn with_capacity<T: Sized>(capacity: usize) -> Self {
        let element_layout = Layout::new::<T>();
        let (block_layout, element_stride) = element_layout.repeat(capacity).unwrap();

        let data = unsafe { std::alloc::alloc_zeroed(block_layout) };

        let valid_elements = BitBuffer::new(capacity);

        Self {
            element_layout,
            element_stride,
            valid_elements,
            capacity,
            data,
            drop_fn: |ptr| unsafe {
                let val = ptr.cast::<T>().read();
                drop(val);
            },
        }
    }

    pub unsafe fn get<T: Sized>(&self, at: usize) -> Option<&T> {
        if at >= self.capacity || !self.valid_elements.get(at)? {
            return None;
        }

        let offset = self.element_stride * at;
        self.data.add(offset).cast::<T>().as_ref()
    }

    pub unsafe fn get_mut<T: Sized>(&self, at: usize) -> Option<&mut T> {
        if at >= self.capacity {
            return None;
        }

        let offset = self.element_stride * at;
        self.data.add(offset).cast::<T>().as_mut()
    }

    pub unsafe fn as_slice<T: Sized>(&self) -> &[T] {
        std::slice::from_raw_parts(self.data.cast::<T>(), self.capacity)
    }

    pub unsafe fn as_slice_mut<T: Sized>(&mut self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.data.cast::<T>(), self.capacity)
    }

    pub unsafe fn insert<T: Sized>(&mut self, at: usize, value: T) -> Option<T> {
        if at >= self.capacity {
            panic!(
                "insertion index (is {at}) must be < than size (is {})",
                self.capacity
            );
        }

        let offset = self.element_stride * at;

        let ptr = self.data.add(offset).cast::<T>();

        let prev_value = if let Some(true) = self.valid_elements.get(at) {
            Some(ptr.read())
        } else {
            None
        };

        ptr.write(value);

        if prev_value.is_none() {
            self.valid_elements.set(at, true);
        }

        prev_value
    }

    /// Expands buffer to `new_capacity`
    ///
    /// If the new capacity is less or equal to the capacity of the buffer
    /// this function does nothing.
    ///
    /// Returns `true` if buffer got expanded
    #[allow(unused)]
    pub fn grow(&mut self, new_capacity: usize) -> bool {
        if self.capacity >= new_capacity {
            return false;
        }

        let (layout, _) = self.element_layout.repeat(self.capacity).unwrap();
        let new_size = self.element_stride * new_capacity;
        let data = unsafe { std::alloc::realloc(self.data, layout, new_size) };
        self.data = data;
        self.capacity = new_capacity;

        true
    }

    pub unsafe fn remove<T>(&mut self, index: usize) -> Option<T> {
        let prev_value = if let Some(true) = self.valid_elements.get(index) {
            let offset = self.element_stride * index;

            let ptr = self.data.add(offset).cast::<T>();
            Some(ptr.read())
        } else {
            None
        };

        self.valid_elements.set(index, false);

        prev_value
    }

    pub fn contains(&self, index: usize) -> bool {
        self.valid_elements.get(index).is_some_and(|val| val)
    }

    pub unsafe fn iter<'a, T: 'a>(&'a self) -> impl Iterator<Item = &'a T> {
        let valid_index_iter = self
            .valid_elements
            .iter()
            .enumerate()
            .filter_map(|(i, val)| val.then(|| i));

        ErasedSparseIter {
            ptr: NonNull::new_unchecked(self.data.cast::<T>()),
            valid_index_iter,
            _marker: PhantomData,
        }
    }

    pub unsafe fn iter_mut<'a, T: 'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> {
        let valid_index_iter = self.valid_indices();

        ErasedSparseIterMut {
            ptr: NonNull::new_unchecked(self.data.cast::<T>()),
            valid_index_iter,
            _marker: PhantomData,
        }
    }

    fn valid_indices<'a>(&'a self) -> impl Iterator<Item = usize> + use<'a> {
        self.valid_elements
            .iter()
            .enumerate()
            .filter_map(|(i, val)| val.then(|| i))
    }
}

impl Drop for ErasedSparseArray {
    fn drop(&mut self) {
        self.valid_indices().for_each(|i| {
            let offset = self.element_stride * i;
            unsafe { (self.drop_fn)(self.data.add(offset).cast()) }
        })
    }
}

pub struct ErasedSparseIter<'a, T, I> {
    ptr: NonNull<T>,
    valid_index_iter: I,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, I> Iterator for ErasedSparseIter<'a, T, I>
where
    T: Sized,
    I: Iterator<Item = usize>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.valid_index_iter.next()?;
        unsafe { Some(self.ptr.add(index).as_ref()) }
    }
}

pub struct ErasedSparseIterMut<'a, T, I> {
    ptr: NonNull<T>,
    valid_index_iter: I,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, I> Iterator for ErasedSparseIterMut<'a, T, I>
where
    T: Sized,
    I: Iterator<Item = usize> + 'a,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.valid_index_iter.next()?;
        unsafe { Some(self.ptr.add(index).as_mut()) }
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, ops::Deref, rc::Rc, sync::atomic::AtomicI32};

    use super::{ErasedSparseArray, INITIAL_CAPACITY};

    #[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SimpleStruct(usize);

    #[test]
    fn create_erased_sparse_array() {
        let _buf = ErasedSparseArray::new::<SimpleStruct>();
    }

    #[test]
    #[should_panic]
    fn return_none_on_invalid_index() {
        let buf = ErasedSparseArray::new::<SimpleStruct>();

        unsafe { buf.get::<SimpleStruct>(0).unwrap() };
    }

    #[test]
    fn insert_element() {
        let mut arr = ErasedSparseArray::new::<SimpleStruct>();

        let first: &SimpleStruct = unsafe {
            arr.insert(0, SimpleStruct(69));
            arr.get(0).unwrap()
        };

        assert_eq!(first, &SimpleStruct(69));
    }

    #[test]
    #[should_panic]
    fn insert_element_out_of_bounds() {
        let mut arr = ErasedSparseArray::new::<SimpleStruct>();

        unsafe { arr.insert(INITIAL_CAPACITY, SimpleStruct::default()) };
    }

    #[test]
    fn iter_elements() {
        let mut arr = ErasedSparseArray::with_capacity::<SimpleStruct>(6);
        (0..6).filter(|i| i % 2 == 1).for_each(|i| {
            unsafe { arr.insert(i, SimpleStruct(i)) };
        });

        let result = unsafe { arr.iter::<SimpleStruct>().collect::<Vec<_>>() };
        assert_eq!(
            result,
            vec![&SimpleStruct(1), &SimpleStruct(3), &SimpleStruct(5)]
        )
    }

    #[test]
    fn drop_items() {
        #[derive(Debug)]
        struct Droppable(Rc<RefCell<i32>>);

        impl Drop for Droppable {
            fn drop(&mut self) {
                *self.0.borrow_mut() -= 1;
            }
        }

        let observer = Rc::new(RefCell::new(5));
        let mut arr = ErasedSparseArray::with_capacity::<Droppable>(5);

        (0..5).for_each(|i| unsafe {
            arr.insert(i, Droppable(observer.clone()));
        });

        drop(arr);

        assert_eq!(observer.borrow().deref(), &0);
    }
}
