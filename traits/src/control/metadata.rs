//! TODO!

use core::any::{Any, TypeId};
use core::convert::{AsRef, TryInto};
use core::hash::{Hasher, Hash};
use core::time::Duration;
use core::fmt::Display;
#[allow(deprecated)] use core::hash::SipHasher; // TODO: this is deprecated (but the replacement isn't available without std).

use lc3_isa::util::MemoryDump;
use lc3_isa::Word;

use serde::{Deserialize, Serialize};

// TODO: `ProgramID` and `ProgramMetadata` should maybe move into lc3-isa. Or we
// should spin off an lc3-program crate (or have an assembler crate) that has
// everything in `isa/src/misc` and `ProgramID` + `ProgramMetadata`.

// TODO: Identifier should probably move too, but I'm not sure to where.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgramId {
    Known { hash: u64 },
    Unknown,
}

impl Default for ProgramId {
    fn default() -> Self {
        ProgramId::Unknown
    }
}

impl ProgramId {
    // Can't be const until const traits arrive (`Hasher`).
    pub /*const*/ fn new(program: &MemoryDump) -> Self {
        #[allow(deprecated)]
        let mut hasher = SipHasher::new();

        // It'd be nice to do &[u16] -> &[u8] and call `hasher.write(...)` and
        // ditch the `for_each` but alas.
        // program.for_each(|w| hasher.write_u16(w));

        // Actually, we can do this which I'll call good enough:
        Word::hash_slice(&**program, &mut hasher);

        Self::Known { hash: hasher.finish() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ProgramMetadata {
    pub name: LongIdentifier,
    pub id: ProgramId,
    /// Time the program was modified in seconds since the Unix epoch.
    pub last_modified: u64,
}

impl ProgramMetadata {
    pub /*const*/ fn new(program: &MemoryDump, modified: Duration) -> Self {
        Self {
            id: ProgramId::new(program),
            last_modified: modified.as_secs()
        }
    }

    pub /*const*/ fn from<P: Into<MemoryDump>>(program: P, modified: Duration) -> Self {
        Self::new(&program.into(), modified)
    }

    pub fn set_last_modified(&mut self, modified: Duration) {
        self.last_modified = modified.as_secs()
    }
}

using_std! {
    // SystemTime instead of Instant since we don't really care about
    // monotonicity.
    use std::time::SystemTime;

    impl ProgramMetadata {
        pub /*const*/ fn new_modified_now(program: &MemoryDump) -> Self {
            Self::new(program, Duration::from_secs(0)).now()
        }

        pub /*const*/ fn from_modified_now<P: Into<MemoryDump>>(program: P) -> Self {
            Self::from(program, Duration::from_secs(0)).now()
        }

        pub fn now(mut self) -> Self {
            self.updated_now();
            self
        }

        pub fn updated_now(&mut self) {
            self.last_modified = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("System time to be later than 1970-01-01 00:00:00 UTC")
                .as_secs();
        }

        pub fn modified_on(&mut self, time: SystemTime) {
            self.last_modified = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("System time to be later than 1970-01-01 00:00:00 UTC")
                .as_secs();
        }
    }
}


// If we had better const functions (+ typenum) or const generics (and better
// const functions — mainly just loops and ranges) we wouldn't need two
// separate types here.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LongIdentifier([u8; 12]);

impl Default for LongIdentifier {
    fn default() -> Self {
        Self::new_from_str("unknown!").unwrap()
    }
}

impl LongIdentifier {
    pub fn new(name: [u8; 12]) -> Result<Self, ()> {
        if !name.iter().all(|c| c.is_ascii()) {
            Err(())
        } else {
            Ok(Self(name))
        }
    }

    pub fn new_from_str(name: &str) -> Result<Self, ()> {
        Self::new(name.as_bytes().try_into().map_err(|_| ())?)
    }

    pub fn new_truncated_padded(name: &str) -> Result<Self, ()> {
        // let a: [u8; 12] = name.chars().chain(core::iter::repeat(' ')).take(12).collect();
        // let arr = [' ' as u8; 12];

        // Self::new_from_str()

        // Err(())

        if let
    }
}

impl Display for LongIdentifier {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{}", self.as_ref())
    }
}

impl AsRef<str> for LongIdentifier {
    fn as_ref(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier([u8; 4]);

impl Identifier {
    pub fn new(name: [u8; 4]) -> Result<Self, ()> {
        for c in name.iter() {
            if !c.is_ascii() {
                return Err(())
            }
        }

        Ok(Self(name))
    }

    pub fn new_from_str(name: &str) -> Result<Self, ()> {
        // if name.len() != 4 {
        //     Err(())
        // }

        Self::new(name.as_bytes().try_into().map_err(|_| ())?)
    }

    pub const fn new_that_crashes_on_invalid_inputs(name: [u8; 4]) -> Self {
        // `is_ascii` == `*c & 128 == 0`
        let canary: [(); 1] = [()];

        // check that the high bit isn't set:
        canary[(name[0] & 128) as usize];
        canary[(name[1] & 128) as usize];
        canary[(name[2] & 128) as usize];
        canary[(name[3] & 128) as usize];

        Self(name)
    }

    pub const fn new_from_str_that_crashes_on_invalid_inputs(name: &str) -> Self {
        let slice = name.as_bytes();

        let canary: [(); 1] = [()];
        let input_length_is_not_four = canary;

        // check that the input length isn't anything other than 4
        input_length_is_not_four[slice.len() ^ 4];

        Self::new_that_crashes_on_invalid_inputs([
            slice[0],
            slice[1],
            slice[2],
            slice[3],
        ])
    }
}

impl Display for Identifier {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{}", self.as_ref())
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
// extra, optional traits
pub struct Capabilities {
    pub storage: bool,
    pub display: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub current_program_metadata: ProgramMetadata,
    pub capabilities: Capabilities,
    pub source_type_id: u64,
    pub source_name: Identifier,
    pub proxies: [Option<Identifier>; 5]
}

impl DeviceInfo {
    pub fn new(metadata: ProgramMetadata, capabilities: Capabilities, type_id: TypeId, name: Identifier, proxies: [Option<Identifier>; 5]) -> Self {
        Self {
            current_program_metadata: metadata,
            capabilities,
            source_type_id: type_id.t(),
            source_name: name,
            proxies
        }
    }

    pub fn add_proxy(mut self, proxy: Identifier) -> Result<Self, Self> {
        if let Some(idx) = self.proxies.iter().enumerate().filter(|(_, p)| p.is_none()).map(|(idx, _)| idx).next() {
            self.proxies[idx] = Some(proxy);

            Ok(self)
        } else {
            Err(self)
        }
    }
}

// We could use the below and some of serde's options to trick serde into
// serializing/deserializing `TypeId`s, but since they really are not portable
// across platforms we won't do this.
//
// If you want to turn the u64 we give you into a `TypeId`, you'll have to do
// the crimes yourself.
struct U64Extractor(Option<u64>);

impl Hasher for U64Extractor {
    fn finish(&self) -> u64 { self.0.unwrap() }
    fn write(&mut self, _: &[u8]) { unreachable!() }
    fn write_u64(&mut self, i: u64) { self.0 = Some(i) }
}

pub trait TypeIdExt: Hash {
    fn t(&self) -> u64;
}

impl TypeIdExt for TypeId {
    fn t(&self) -> u64 {
        let mut h = U64Extractor(None);

        self.hash(&mut h);
        h.finish()
    }
}

pub trait AnyExt: Any {
    fn type_id_u64(&self) -> u64 {
        self.type_id().t()
    }
}

impl<T: Any> AnyExt for T {}
