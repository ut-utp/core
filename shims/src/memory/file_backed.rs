//! A file backed [`Memory` trait](lc3_traits::memory::Memory) implementation called
//! [`FileBackedMemoryShim`](memory::FileBackedMemoryShim).
//! (TODO!)

use std::convert::TryInto;
use std::fs::File;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use lc3_isa::{Addr, Word, ADDR_SPACE_SIZE_IN_BYTES, ADDR_SPACE_SIZE_IN_WORDS, MEM_MAPPED_START_ADDR};
use lc3_isa::util::MemoryDump;
use lc3_traits::memory::Memory;
use lc3_traits::control::Control;
use lc3_traits::control::metadata::{LongIdentifier, ProgramMetadata};
use lc3_traits::control::load::{PageIndex, Index as PIdx, PageAccess, PAGE_SIZE_IN_WORDS, LoadMemoryProgress, LoadMemoryDumpError, load_whole_memory_dump};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::error::MemoryShimError;

#[derive(Clone)]
pub struct FileBackedMemoryShim {
    pub path: PathBuf,
    pub mem: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    pub current: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    pub metadata: ProgramMetadata,
}

impl Default for FileBackedMemoryShim {
    fn default() -> Self {
        Self::new("lc3.mem")
    }
}

impl FileBackedMemoryShim {
    pub fn with_initialized_memory<P: AsRef<Path>>(
        path: P,
        memory: MemoryDump,
    ) -> Self {
        let path = path.as_ref().to_path_buf();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| LongIdentifier::new_truncated_padded(n).ok())
            .unwrap_or_default();

        Self {
            path,
            mem: *memory.clone(),
            current: *memory,
            metadata: ProgramMetadata::new_modified_now(name, &memory),
        }
    }

    pub fn from_existing_file<P: AsRef<Path>>(path: &P) -> Result<Self, MemoryShimError> {
        let mut memory: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0u16; ADDR_SPACE_SIZE_IN_WORDS];
        let modified_time = read_from_file(path, &mut memory)?;

        let mut mem = Self::with_initialized_memory(path, memory.into());
        mem.metadata.modified_on(modified_time);

        Ok(mem)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self::with_initialized_memory(path, [0u16; ADDR_SPACE_SIZE_IN_WORDS].into())
    }

    // Note: this writes the changes in the 'temporary' copy of memory and
    // completely ignores the persistent copy!
    //
    // If you wish to make changes to the persistent copy, use the commit_page
    // function that's part of the memory trait.
    //
    // In flushing the changes, this will also replace the persistent copy
    // with the temporary copy (as well as write it into the file).
    pub fn flush_all_changes(&mut self) -> Result<(), MemoryShimError> {
        self.mem = self.current.clone();
        write_to_file(&self.path, &self.mem)?;
        self.metadata.updated_now();

        Ok(())
    }

    pub fn change_file<P: AsRef<Path>>(&mut self, path: P) {
        self.path = path.as_ref().to_path_buf();
    }
}

impl From<FileBackedMemoryShim> for MemoryDump {
    fn from(mem: FileBackedMemoryShim) -> MemoryDump {
        mem.mem.into()
    }
}

impl Index<Addr> for FileBackedMemoryShim {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        &self.current[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl IndexMut<Addr> for FileBackedMemoryShim {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        &mut self.current[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl Memory for FileBackedMemoryShim {
    // Note: doesn't wipe out the corresponding page from the temporary memory.
    // Must call reset for that to happen.
    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]) {
        assert!(page_idx < MEM_MAPPED_START_ADDR.page_idx());

        // TODO: right now we have three copies: the file, the in memory
        // persistent copy, and the in memory temporary copy. We only actually
        // need two; the file and the in memory persistent copy should be the
        // same which makes the in memory persistent copy redundant.
        //
        // Once we figure out how to write to only a page of memory in a file
        // (and how to account for edge cases like the file changing underneath
        // us), we can get rid of the in memory persistent copy (`self.mem`).
        self.mem[PIdx(page_idx).as_index_range()].copy_from_slice(page);

        write_to_file(&self.path, &self.mem).unwrap(); // note: crashes (TODO?)
        self.metadata.updated_now();
    }

    fn reset(&mut self) {
        // TODO: once we do the above this will become a file read call
        self.current = self.mem.clone();
    }

    fn get_program_metadata(&self) -> ProgramMetadata {
        self.metadata.clone()
    }

    fn set_program_metadata(&mut self, metadata: ProgramMetadata) {
        self.metadata = metadata
    }
}

pub(super) fn read_from_file<P: AsRef<Path>>(
    path: P,
    mem: &mut [Word; ADDR_SPACE_SIZE_IN_WORDS],
) -> Result<SystemTime, MemoryShimError> {
    let mut file = File::open(path)?;

    let length = file.metadata()?.len();
    if length != TryInto::<u64>::try_into(ADDR_SPACE_SIZE_IN_BYTES).unwrap() {
        return Err(MemoryShimError::IncorrectlySizedFile(length));
    }

    file.read_u16_into::<LittleEndian>(mem)?;

    Ok(file.metadata()?.modified()?)
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

impl FileBackedMemoryShim {
    // Note: loads the persistent copy (mem) and not the staging copy or the file.
    pub fn load<C: Control, P: LoadMemoryProgress>(&self, sim: &mut C, progress: Option<&P>) -> Result<(), LoadMemoryDumpError> {
        load_whole_memory_dump(sim, &self.mem.into(), progress)?;
        sim.set_program_metadata(self.metadata.clone());

        Ok(())
    }
}
