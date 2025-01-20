use std::{alloc::Layout, marker::PhantomData, ops::Index, ptr::addr_of, slice::SliceIndex};

pub struct BitVec {
    data: BitVecData,
    byte_count: usize,
}

union BitVecData {
    _ptr: *mut u8,
    _u64: u64,
}

#[allow(dead_code)]
impl BitVec {
    pub fn new_short() -> Self {
        Self::new(size_of::<BitVecData>())
    }

    pub fn new(byte_count: usize) -> Self {
        let data = init_data(byte_count);

        Self { byte_count, data }
    }

    pub fn set_bit(&mut self, index: usize, value: bool) {
        assert!(index < self.byte_count * 8);

        let (byte_index, bit_index) = bit_index(index);

        let data = unsafe { self.data_ptr_mut().add(byte_index) };

        unsafe {
            if value {
                *data |= 1 << bit_index;
            } else {
                *data &= !(1 << bit_index);
            }
        }
    }

    pub fn toggle_bit(&mut self, index: usize) {
        assert!(index < self.byte_count * 8);

        let (byte_index, bit_index) = bit_index(index);

        let byte = unsafe { self.data_ptr_mut().add(byte_index) };

        unsafe {
            *byte ^= 1 << bit_index;
        }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        let (byte_index, bit_index) = bit_index(index);

        if byte_index >= self.byte_count {
            None
        } else {
            let value = unsafe {
                let byte = self.data_ptr().add(byte_index);
                *byte & (1 << bit_index) != 0
            };

            Some(value)
        }
    }

    pub fn iter(&self) -> BitVecIter {
        BitVecIter {
            index: 0,
            data: self.data_slice(),
        }
    }

    fn data_slice_mut(&mut self) -> &mut [u8] {
        let ptr = self.data_ptr() as *mut u8;

        unsafe { std::slice::from_raw_parts_mut(ptr, self.byte_count) }
    }

    fn data_slice(&self) -> &[u8] {
        let ptr = self.data_ptr() as *const u8;

        unsafe { std::slice::from_raw_parts(ptr, self.byte_count) }
    }

    fn data_ptr_mut(&mut self) -> *mut u8 {
        let ptr = if self.byte_count <= size_of::<BitVecData>() {
            unsafe { addr_of!(self.data._u64) as *mut u8 }
        } else {
            unsafe { self.data._ptr }
        };
        ptr
    }

    fn data_ptr(&self) -> *const u8 {
        let ptr = if self.byte_count <= size_of::<BitVecData>() {
            unsafe { addr_of!(self.data._u64) as *const u8 }
        } else {
            unsafe { self.data._ptr }
        };
        ptr
    }
}

impl Clone for BitVec {
    fn clone(&self) -> Self {
        let mut dst = init_data(self.byte_count);

        if self.byte_count <= size_of::<BitVecData>() {
            dst._u64 = unsafe { self.data._u64 };
        } else {
            unsafe {
                dst._ptr.copy_from(self.data._ptr, self.byte_count);
            }
        }

        Self {
            data: dst,
            byte_count: self.byte_count.clone(),
        }
    }
}

impl Drop for BitVec {
    fn drop(&mut self) {
        if self.byte_count > size_of::<BitVecData>() {
            unsafe {
                std::alloc::dealloc(
                    self.data._ptr,
                    Layout::from_size_align_unchecked(self.byte_count, align_of::<BitVecData>()),
                );
            }
        }
    }
}

pub struct BitVecIter<'a> {
    index: usize,
    data: &'a [u8],
}

impl<'a> Iterator for BitVecIter<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.data.get(self.index)?;
        self.index += 1;

        Some(val)
    }
}

fn init_data(byte_count: usize) -> BitVecData {
    let data = if byte_count <= size_of::<BitVecData>() {
        BitVecData { _u64: 0 }
    } else {
        let ptr = vec![0; byte_count].into_boxed_slice();
        let ptr = Box::into_raw(ptr) as *mut u8;
        BitVecData { _ptr: ptr }
    };
    data
}

fn bit_index(index: usize) -> (usize, usize) {
    let byte_index = index / 8;
    let bit_index = index % 8;

    (byte_index, bit_index)
}

#[cfg(test)]
mod test {
    use super::BitVec;

    #[test]
    fn should_create_bit_vec_with_stack_allocation() {
        let bitvec = BitVec::new(16);
    }

    #[test]
    fn should_create_bit_vec_with_heap_allocation() {
        let bitvec = BitVec::new(128);
    }

    #[test]
    fn should_clone_bitvec_with_heap_allocation() {
        let bitvec = BitVec::new(128);
        let other = bitvec.clone();
    }

    #[test]
    fn should_clone_bitve_with_stack_allocaiton() {
        let bitvec = BitVec::new(16);
        let other = bitvec.clone();
    }

    #[test]
    fn should_set_bit() {
        let mut bitvec = BitVec::new(1);
        bitvec.set_bit(3, true);

        assert!(bitvec.get(3) == Some(true));
    }

    #[test]
    fn should_iter_bitvec() {
        let mut bitvec = BitVec::new(4);
        bitvec.set_bit(0, true);

        let collected = bitvec.iter().collect::<Vec<_>>();
        println!("collected: {collected:?}");
        assert!(collected == vec![&1, &0, &0, &0])
    }
}
