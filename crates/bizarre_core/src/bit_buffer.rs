use std::alloc::Layout;

const SHORT_BUFFER_SIZE: usize = size_of::<BitBufferData>();

pub struct BitBuffer {
    size: usize,
    data: BitBufferData,
}

impl BitBuffer {
    pub const MAX_WIDTH: usize = (isize::MAX >> 25) as usize;

    pub fn new_short() -> Self {
        Self::new(SHORT_BUFFER_SIZE * 8)
    }

    pub fn new(width: usize) -> Self {
        debug_assert!(width <= Self::MAX_WIDTH, "Cannot create a `BitBuffer` with width more than {} (trying to create with `width` = {width})", Self::MAX_WIDTH);

        let size = width_to_size(width);
        let size = size.max(SHORT_BUFFER_SIZE);
        let data = BitBufferData::new(size);

        Self { size, data }
    }

    pub fn data(&self) -> &[u8] {
        unsafe { self.data_ptr().as_ref().unwrap() }
    }

    fn data_mut(&mut self) -> &mut [u8] {
        unsafe { self.data_ptr_mut().as_mut().unwrap() }
    }

    pub fn is_short(&self) -> bool {
        self.size <= SHORT_BUFFER_SIZE
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn width(&self) -> usize {
        self.size * 8
    }

    pub fn set(&mut self, at: usize, value: bool) {
        debug_assert!(
            at / 8 < self.size,
            "trying to set bit at index {at} (len is {})",
            self.width()
        );

        let (byte, bit) = byte_bit_index(at);
        let data = self.data_mut();

        if value {
            data[byte] |= 1 << bit;
        } else {
            data[byte] &= !(1 << bit);
        }
    }

    pub fn toggle(&mut self, at: usize) {
        debug_assert!(at / 8 < self.size);

        let byte = at / 8;
        let bit = at % 8;

        self.data_mut()[byte] ^= 1 << bit;
    }

    pub fn clear(&mut self) {
        if self.is_short() {
            self.data.short_data = 0;
        } else {
            self.data_mut().fill(0);
        }
    }

    pub fn all_zeroes(&mut self) -> bool {
        if self.is_short() {
            unsafe { self.data.short_data == 0 }
        } else {
            self.data().iter().all(|val| *val == 0)
        }
    }

    pub fn get(&self, at: usize) -> Option<bool> {
        if at / 8 >= self.size {
            return None;
        }

        let byte = at / 8;
        let bit = at % 8;

        Some(self.data()[byte] & (1 << bit) != 0)
    }

    pub fn expand_to(&mut self, new_width: usize) {
        debug_assert!(
            new_width <= Self::MAX_WIDTH,
            "Cannot expand a `BitBuffer` to width more than {} bits (trying to expand to {new_width} bits)",
            Self::MAX_WIDTH
        );

        let new_size = width_to_size(new_width);

        if new_size <= SHORT_BUFFER_SIZE {
            return;
        }

        if self.is_short() {
            let data = BitBufferData::new(new_size);

            unsafe {
                data.ptr
                    .cast::<u8>()
                    .copy_from_nonoverlapping(self.data_ptr() as *const u8, SHORT_BUFFER_SIZE);
            };

            self.data = data;
            self.size = new_size;
        } else {
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.size, 1);
                let ptr = std::alloc::realloc(self.data.ptr as *mut u8, layout, new_size);
                let ptr = std::ptr::slice_from_raw_parts_mut(ptr, new_size);
                (*ptr)[self.size..].fill(0);
                self.data.ptr = ptr;
            }

            self.size = new_size;
        }
    }

    pub fn copy_from(&mut self, other: &Self) {
        let size = self.size.min(other.size);
        unsafe {
            self.data_ptr_mut()
                .cast::<u8>()
                .copy_from_nonoverlapping(other.data_ptr().cast(), size)
        };
    }

    pub fn iter<'a>(&'a self) -> BitBufferIter<'a> {
        BitBufferIter {
            data: self.data(),
            index: 0,
        }
    }

    fn data_ptr_mut(&mut self) -> *mut [u8] {
        self.data_ptr().cast_mut()
    }

    fn data_ptr(&self) -> *const [u8] {
        if self.is_short() {
            unsafe {
                std::slice::from_raw_parts(
                    (&raw const self.data.short_data) as *const u8,
                    SHORT_BUFFER_SIZE,
                )
            }
        } else {
            unsafe { self.data.ptr }
        }
    }
}

fn byte_bit_index(at: usize) -> (usize, usize) {
    let byte = at / 8;
    let bit = at % 8;
    (byte, bit)
}

fn width_to_size(width: usize) -> usize {
    let size = width / 8 + 1.min(width % 8);
    size
}

union BitBufferData {
    ptr: *mut [u8],
    short_data: u128,
}

impl BitBufferData {
    pub fn new(size: usize) -> Self {
        if size <= SHORT_BUFFER_SIZE {
            Self { short_data: 0 }
        } else {
            let layout = Layout::from_size_align(size, 1).unwrap();
            let ptr = unsafe {
                let ptr = std::alloc::alloc_zeroed(layout);
                std::ptr::slice_from_raw_parts_mut(ptr, size)
            };
            Self { ptr }
        }
    }
}

pub struct BitBufferIter<'a> {
    index: usize,
    data: &'a [u8],
}

impl<'a> Iterator for BitBufferIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let (byte, bit) = byte_bit_index(self.index);
        let result = if byte >= self.data.len() {
            None
        } else {
            let value = self.data[byte] & (1 << bit) != 0;
            Some(value)
        };

        self.index += 1;

        result
    }
}

#[cfg(test)]
mod test {
    use crate::bit_buffer::SHORT_BUFFER_SIZE;

    use super::BitBuffer;

    #[test]
    fn should_allocate_short() {
        let buf = BitBuffer::new_short();
        assert!(buf.is_short());
        assert_eq!(buf.size, SHORT_BUFFER_SIZE);

        let buf = BitBuffer::new(2);
        assert!(buf.is_short());
        assert_eq!(buf.size, SHORT_BUFFER_SIZE);
    }

    #[test]
    fn should_allocate_long() {
        let buf = BitBuffer::new(SHORT_BUFFER_SIZE * 8 + 1);
        assert!(!buf.is_short());
        assert_eq!(buf.size, SHORT_BUFFER_SIZE + 1);

        let width = BitBuffer::MAX_WIDTH;
        let buf = BitBuffer::new(width);
        assert_eq!(buf.size(), width / 8 + 1.min(width % 8));
    }

    #[test]
    fn shoule_set_bit_in_short() {
        let mut buf = BitBuffer::new_short();
        buf.set(0, true);

        assert_eq!(buf.get(0), Some(true));
    }

    #[test]
    fn should_set_bit_in_long() {
        let mut buf = BitBuffer::new(1024);
        buf.set(512, true);

        assert_eq!(buf.get(512), Some(true));
    }

    #[test]
    fn should_toggle_bit_in_long() {
        let mut buf = BitBuffer::new(1024);

        buf.toggle(512);
        assert_eq!(buf.get(512), Some(true));
        buf.toggle(512);
        assert_eq!(buf.get(512), Some(false));
    }

    #[test]
    fn should_return_none_if_out_of_bounds() {
        let buf = BitBuffer::new_short();
        assert_eq!(buf.get(512), None);

        let buf = BitBuffer::new(256);
        assert_eq!(buf.get(512), None);
    }

    #[test]
    fn should_expand_from_short_to_long() {
        let mut buf = BitBuffer::new_short();
        buf.toggle(4);

        buf.expand_to(255);

        assert_eq!(buf.size(), 32);
        assert_eq!(buf.get(4), Some(true));
    }

    #[test]
    fn should_clear_short() {
        let mut buf = BitBuffer::new_short();

        buf.toggle(16);
        assert!(!buf.all_zeroes());
        buf.clear();
        assert!(buf.all_zeroes());
    }

    #[test]
    fn should_clear_long() {
        let mut buf = BitBuffer::new(256);

        buf.toggle(129);
        assert!(!buf.all_zeroes());
        buf.clear();
        assert!(buf.all_zeroes());
    }

    #[test]
    fn should_expand_from_long_to_long() {
        let mut buf = BitBuffer::new(256);
        buf.toggle(128);

        buf.expand_to(1024);
        assert_eq!(buf.size(), 128);
        assert_eq!(buf.get(128), Some(true));
    }

    #[test]
    fn should_properly_report_zeroes_in_short_buffer() {
        let mut buf = BitBuffer::new_short();
        assert_eq!(buf.all_zeroes(), true);
        buf.toggle(12);
        assert_eq!(buf.all_zeroes(), false);
        buf.toggle(12);
        assert_eq!(buf.all_zeroes(), true);
    }

    #[test]
    fn should_properly_report_zeroes_in_long_buffer() {
        let mut buf = BitBuffer::new(256);

        assert_eq!(buf.all_zeroes(), true);
        buf.toggle(128);
        assert_eq!(buf.all_zeroes(), false);
        buf.toggle(128);
        assert_eq!(buf.all_zeroes(), true);
    }
}
