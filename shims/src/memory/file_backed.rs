//! A file backed [`Memory` trait](lc3_traits::memory::Memory) implementation called
//! [`FileBackedMemoryShim`](memory::FileBackedMemoryShim).
//! (TODO!)

use std::convert::TryInto;
use std::fs::File;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};

use lc3_isa::{Addr, Word, ADDR_SPACE_SIZE_IN_BYTES, ADDR_SPACE_SIZE_IN_WORDS};
use lc3_traits::memory::{Memory, MemoryMiscError};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::error::MemoryShimError;

pub struct FileBackedMemoryShim {
    path: PathBuf,
    memory: [Word; ADDR_SPACE_SIZE_IN_WORDS],
}

impl Default for FileBackedMemoryShim {
    fn default() -> Self {
        Self::new("lc3.mem")
    }
}

impl FileBackedMemoryShim {
    pub fn with_initialized_memory<P: AsRef<Path>>(
        path: P,
        memory: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            memory,
        }
    }

    pub fn from_existing_file<P: AsRef<Path>>(path: &P) -> Result<Self, MemoryShimError> {
        let mut memory: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0u16; ADDR_SPACE_SIZE_IN_WORDS];
        read_from_file(path, &mut memory)?;

        Ok(Self::with_initialized_memory(path, memory))
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self::with_initialized_memory(path, [0u16; ADDR_SPACE_SIZE_IN_WORDS])
    }

    fn flush(&mut self) -> Result<(), MemoryShimError> {
        write_to_file(&self.path, &self.memory)
    }
}

impl Index<Addr> for FileBackedMemoryShim {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        &self.memory[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl IndexMut<Addr> for FileBackedMemoryShim {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        &mut self.memory[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl Memory for FileBackedMemoryShim {
    fn commit(&mut self) -> Result<(), MemoryMiscError> {
        self.flush().map_err(|_| MemoryMiscError)
    }
}

pub(super) fn read_from_file<P: AsRef<Path>>(
    path: P,
    mem: &mut [Word; ADDR_SPACE_SIZE_IN_WORDS],
) -> Result<(), MemoryShimError> {
    let mut file = File::open(path)?;

    let length = file.metadata()?.len();
    if length != ADDR_SPACE_SIZE_IN_BYTES.try_into().unwrap() {
        return Err(MemoryShimError::IncorrectlySizedFile(length));
    }

    file.read_u16_into::<LittleEndian>(mem)?;

    Ok(())
}

pub(super) fn write_to_file<P: AsRef<Path>>(
    path: P,
    mem: &[Word; ADDR_SPACE_SIZE_IN_WORDS],
) -> Result<(), MemoryShimError> {
    let mut file = File::create(path)?;

    for word in mem.iter() {
        file.write_u16::<LittleEndian>(*word)?
    }

    file.sync_all()?;
    Ok(())
}
