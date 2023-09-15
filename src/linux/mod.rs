extern crate libc;

use crate::device::{IvshmemDevice, MappedMemory};
use anyhow::bail;
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::fs::{File, OpenOptions};
use std::path::Path;
use anyhow::Result;
use libc::FILE;

pub(crate) struct UnixMemoryMap<'a> {
    memory: &'a mut [u8],
}

impl UnixMemoryMap<'_> {
    pub fn new(path: &Path) -> Result<Self> {
        let path = CString::new(path.to_str().expect("Unable to convert path to CString"))?;
        unsafe {
            let file_descriptor = libc::open(path.as_ptr(), libc::O_RDWR, 0o000);
            if file_descriptor == -1 {
                bail!("Failed to open shared memory.");
            }
            let size = libc::lseek(file_descriptor, 0, libc::SEEK_END) as usize;
            libc::lseek(file_descriptor, 0, libc::SEEK_SET);
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file_descriptor,
                0,
            );
            if ptr == libc::MAP_FAILED {
                bail!("Failed to map shared memory");
            }
            let slice: &mut [u8] = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
            Ok(Self { memory: slice })
        }
    }
}

impl Debug for UnixMemoryMap<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory{{ size: {:?} }}",
            self.memory.len()
        )?;
        Ok(())
    }
}

impl MappedMemory for UnixMemoryMap<'_> {
    #[inline(always)]
    fn ptr(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}

pub fn ivshmem_device<'a>(path: &Path) -> Result<IvshmemDevice> {
    let memory_map = UnixMemoryMap::new(path)?;
    Ok(IvshmemDevice::with_memory(memory_map))
}
