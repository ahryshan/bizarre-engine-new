use std::os::fd::{AsFd, OwnedFd};

use rustix::{fs::ftruncate, shm};

pub struct SharedMemory {
    name: String,
    size: usize,
    fd: OwnedFd,
    o_flags: shm::OFlags,
    mode: shm::Mode,
}

impl SharedMemory {
    pub fn new(
        name: String,
        size: usize,
        flags: shm::OFlags,
        mode: shm::Mode,
    ) -> rustix::io::Result<Self> {
        let fd = shm::open(&name, flags, mode)?;

        ftruncate(fd.as_fd(), size as u64)?;

        Ok(Self {
            name,
            size,
            fd,
            o_flags: flags,
            mode,
        })
    }
}

impl AsFd for SharedMemory {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        shm::unlink(&self.name);
    }
}
