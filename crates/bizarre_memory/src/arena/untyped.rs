use std::alloc::Layout;

pub struct ArenaChunk {
    memory: *mut u8,
    size: usize,
    offset: usize,
}

impl ArenaChunk {
    pub fn new(size: usize) -> Self {
        let memory =
            unsafe { std::alloc::alloc_zeroed(Layout::from_size_align_unchecked(size, 1)) };

        Self {
            memory,
            size,
            offset: 0,
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        if self.size - self.offset <= layout.size() {
            None
        } else {
            let ptr = unsafe { self.memory.add(self.offset) };
            self.offset += layout.size();
            Some(ptr)
        }
    }

    pub fn alloc_unchecked(&mut self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.memory.add(self.offset) };
        self.offset += layout.size();
        ptr
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum ChunkStatus {
    Full,
    HasMemory(usize),
}

pub struct Arena {
    pub(crate) chunks: Vec<ArenaChunk>,
    pub(crate) chunk_size: usize,
    pub(crate) chunk_status: Vec<ChunkStatus>,
}

impl Arena {
    pub fn new(chunk_size: usize) -> Self {
        let chunks = vec![ArenaChunk::new(chunk_size)];
        let chunk_status = vec![ChunkStatus::HasMemory(chunk_size)];

        Self {
            chunks,
            chunk_size,
            chunk_status,
        }
    }

    pub fn alloc<T>(&mut self) -> *mut T {
        let layout = Layout::new::<T>();
        let index = self
            .chunk_status
            .iter()
            .enumerate()
            .find(|(_, c)| {
                if let ChunkStatus::HasMemory(mem) = c {
                    mem >= &layout.size()
                } else {
                    false
                }
            })
            .map(|(index, _)| index);

        if let Some(index) = index {
            let chunk = &mut self.chunks[index];
            let ptr = chunk.alloc_unchecked(layout) as *mut T;
            self.update_chunk_status(index);
            ptr
        } else {
            let index = self.chunks.len();
            self.add_chunk();
            let chunk = self.chunks.last_mut().unwrap();
            let ptr = chunk.alloc_unchecked(layout) as *mut T;
            self.update_chunk_status(index);
            ptr
        }
    }

    pub fn reset(&mut self) {
        self.chunks.iter_mut().for_each(|c| c.reset());
    }

    fn add_chunk(&mut self) {
        self.chunks.push(ArenaChunk::new(self.chunk_size));
        self.chunk_status
            .push(ChunkStatus::HasMemory(self.chunk_size));
    }

    #[inline(always)]
    fn update_chunk_status(&mut self, index: usize) {
        let chunk = &mut self.chunks[index];
        let memory_remained = chunk.size - chunk.offset;
        if memory_remained > 0 {
            self.chunk_status[index] = ChunkStatus::HasMemory(memory_remained);
        } else {
            self.chunk_status[index] = ChunkStatus::Full;
        }
    }
}
