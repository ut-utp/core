//! TODO!

#![macro_use]

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
// everything in `isa/src/misc` and `ProgramId` + `ProgramMetadata`.

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
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    // Can't be const until const traits arrive (`Hasher`).
    pub fn new(program: &MemoryDump) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProgramMetadata {
    pub name: LongIdentifier,
    pub id: ProgramId,
    /// Time the program was modified in seconds since the Unix epoch.
    pub last_modified: u64,
}

impl Default for ProgramMetadata {
    fn default() -> Self {
        Self::empty()
    }
}

impl ProgramMetadata {
    pub const fn empty() -> Self {
        Self {
            name: LongIdentifier::unknown(),
            id: ProgramId::unknown(),
            last_modified: 0,
        }
    }

    pub fn new(
        name: LongIdentifier,
        program: &MemoryDump,
        modified: Duration,
    ) -> Self {
        Self {
            name,
            id: ProgramId::new(program),
            last_modified: modified.as_secs(),
        }
    }

    pub fn from<P: Into<MemoryDump>>(
        name: LongIdentifier,
        program: P,
        modified: Duration,
    ) -> Self {
        Self::new(name, &program.into(), modified)
    }

    pub fn set_last_modified(&mut self, modified: Duration) {
        self.last_modified = modified.as_secs()
    }
}

// TODO: wasm! (we don't have SystemTime on wasm)
using_std! {
    // SystemTime instead of Instant since we don't really care about
    // monotonicity.
    use std::time::SystemTime;

    impl ProgramMetadata {
        pub /*const*/ fn new_modified_now(name: LongIdentifier, program: &MemoryDump) -> Self {
            Self::new(name, program, Duration::from_secs(0)).now()
        }

        pub /*const*/ fn from_modified_now<P: Into<MemoryDump>>(name: LongIdentifier, program: P) -> Self {
            Self::from(name, program, Duration::from_secs(0)).now()
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
#[repr(transparent)]
pub struct LongIdentifier([u8; 8]);

impl LongIdentifier {
    pub const MAX_LEN: usize = 8;
}

impl Default for LongIdentifier {
    fn default() -> Self {
        Self::unknown()
    }
}

impl LongIdentifier {
    pub const fn unknown() -> Self {
        Self::new_from_str_that_crashes_on_invalid_inputs("unknown!")
    }

    pub fn new(name: [u8; Self::MAX_LEN]) -> Result<Self, ()> {
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
        let mut arr = [0; Self::MAX_LEN];

        for (idx, c) in name.chars().take(Self::MAX_LEN).enumerate() {
            if !c.is_ascii() {
                return Err(());
            }

            arr[idx] = c as u8;
        }

        Ok(Self(arr))
    }

    pub const fn new_that_crashes_on_invalid_inputs(
        name: [u8; Self::MAX_LEN],
    ) -> Self {
        // `is_ascii` == `*c & 128 == 0`
        let canary: [(); 1] = [()];

        macro_rules! is_ascii {
            ($($num:literal)*) => {$(
                // check that the high bit isn't set:
                canary[(name[$num] & 128) as usize];
            )*};
        }

        is_ascii!{ 0 1 2 3 4 5 6 7 }

        Self(name)
    }

    pub const fn new_from_str_that_crashes_on_invalid_inputs(
        name: &str,
    ) -> Self {
        let slice = name.as_bytes();

        let canary: [(); 1] = [()];
        let input_length_is_not_eight = canary;

        // check that the input length isn't anything other than 8
        input_length_is_not_eight[slice.len() ^ 8];

        // let [a, b, c, d, e, f, g, h] = *slice;
        // Self::new_that_crashes_on_invalid_inputs([
        //     /*a, b, c, d, e, f, g, h,*/
        // ])

        macro_rules! slice2arr {
            ($arr:ident: $($idx:literal)*) => {[$(
                $arr[$idx],
            )*]};
        }

        Self::new_that_crashes_on_invalid_inputs(
            slice2arr!(slice: 0 1 2 3 4 5 6 7)
        )
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Identifier([u8; 4]);

impl Identifier {
    pub const MAX_LEN: usize = 4;
}

impl Identifier {
    pub fn new(name: [u8; Self::MAX_LEN]) -> Result<Self, ()> {
        for c in name.iter() {
            if !c.is_ascii() {
                return Err(());
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

    pub const fn empty() -> Self {
        Self::new_from_str_that_crashes_on_invalid_inputs("    ")
    }

    pub const fn new_that_crashes_on_invalid_inputs(
        name: [u8; Self::MAX_LEN],
    ) -> Self {
        // `is_ascii` == `*c & 128 == 0`
        let canary: [(); 1] = [()];

        // check that the high bit isn't set:
        canary[(name[0] & 128) as usize];
        canary[(name[1] & 128) as usize];
        canary[(name[2] & 128) as usize];
        canary[(name[3] & 128) as usize];

        Self(name)
    }

    pub const fn new_from_str_that_crashes_on_invalid_inputs(
        name: &str,
    ) -> Self {
        let slice = name.as_bytes();

        let canary: [(); 1] = [()];
        let input_length_is_not_four = canary;

        // check that the input length isn't anything other than 4
        input_length_is_not_four[slice.len() ^ 4];

        Self::new_that_crashes_on_invalid_inputs([
            slice[0], slice[1], slice[2], slice[3],
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
    pub disk: bool,
    pub display: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub pre: Option<Identifier>,
}

impl Default for Version {
    fn default() -> Self {
        Self::empty()
    }
}

impl Version {
    // Same as what the `Default` impl returns, but this function is `const`.
    pub const fn empty() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            pre: None
        }
    }

    pub const fn new(major: u8, minor: u8, patch: u8, pre: Option<Identifier>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre,
        }
    }

    pub const fn major(mut self, major: u8) -> Self {
        self.major = major;
        self
    }

    pub const fn minor(mut self, minor: u8) -> Self {
        self.minor = minor;
        self
    }

    pub const fn patch(mut self, patch: u8) -> Self {
        self.patch = patch;
        self
    }

    pub const fn pre(mut self, pre: Identifier) -> Self {
        self.pre = Some(pre);
        self
    }

    pub fn get_pre(&self) -> Option<&str> {
        if let Some(pre) = self.pre.as_ref() {
            let mut idx = Identifier::MAX_LEN - 1;

            // The entire Identifier can't be empty; Cargo doesn't allow it.
            // Even so, we'll be safe in case someone decided to not use the
            // Cargo constructor.
            while pre.0[idx] == b" "[0] {
                if idx == 0 { return None; }

                idx -= 1;
            }

            Some(core::str::from_utf8(&pre.0[0..=idx]).unwrap())
        } else {
            None
        }
    }

    pub const fn pre_from_str_that_crashes_on_invalid_inputs(
        self,
        pre: &str,
    ) -> Self {
        self.pre(Identifier::new_from_str_that_crashes_on_invalid_inputs(pre))
    }

    /// This takes arguments because if we ask for the env vars in compiled
    /// code, we'll get the values of those env vars for this crate which is not
    /// very useful.
    ///
    /// See the [`version_from_crate!()`](version_from_crate) macro.
    #[rustfmt::skip]
    pub const fn from_cargo_env_vars<'a>(
        major: &'a str,
        minor: &'a str,
        patch: &'a str,
        pre: &'a str,
    ) -> Self {
        // Cargo is very good about actually making people follow semver.
        // Major, minor, and patch versions are all _required_ and pre-release
        // version tags are picked up. Build version tags are allowed but not
        // reported through env vars which is fine; we don't use them anyways.
        //
        // Parsing seems pretty good too; only numbers for major, minor, patch;
        // Cargo understands that pre-release versions have to come before build
        // versions and can't contain a +. However this: "ei-ewori-w" is a valid
        // pre-release version which is a little questionable but is in
        // accordance with the grammar in the semver 2.0.0 spec.
        //
        // Mercifully, Cargo also yells at you if you try to put
        // non-alphanumeric characters into the pre-release version (also in
        // accordance with the semver spec) which makes our job a lot easier.
        // It also means we could use something more compact than an Identifier
        // (has encodings for all ASCII characters) for the pre-release version,
        // but there isn't really a point.
        //
        // Blessedly, Cargo also yells at you for leading zeros.
        //
        // So, here are the reasons why this might fail:
        //   - major/minor/patch versions greater than 255 (seems unlikely)
        //   - pre-release version longer than the 4 characters allowed by the
        //     `Identifier` type.

        // Now we've got the exciting job of having to parse strings in u8's
        // in const contexts (i.e. with no loops, conditionals, or any real
        // support from std).
        const fn ver_str_to_u8<'a>(v: &'a str) -> u8 {
            // Since Cargo has made sure these are just numbers, we can assume
            // one byte per number and safely treat this as a bunch of ASCII
            // bytes.
            let bytes = v.as_bytes();

            // If we have more than 3 characters, we can bail right away:
            let version_component_is_too_long = [()];
            version_component_is_too_long[(bytes.len() > 3) as usize];

            // Zero characters should not be possible. So, we're left with 1, 2,
            // or 3. The trouble is that we can't do anything _conditionally_ on
            // the length and we also can't ask for characters that don't exist
            // without crashing. So what to do?
            //
            // Time for some cunning and guile:

            // Let's try this another way. Conditional execution is a no go, so
            // let's instead just use a dummy value for the 10s and 100s digit
            // when they're not actually there (in the below we just use the
            // ones digit for them).
            let len = bytes.len() - 1; // -1 catches the impossible empty case.
            let one = [0, 0, 0];
            let two = [0, 1, 1];
            let tre = [0, 0, 2];

            let padded = [
                bytes[one[len]] - b'0',
                bytes[two[len]] - b'0',
                bytes[tre[len]] - b'0',
            ];

            // ['1', '2', '3'] => h(0), t(1), o(2)
            // ['2', '3'     ] => h(-), t(0), o(1)
            // ['3'          ] => h(-), t(-), o(0)

            // And then we go use zero for the places that aren't actually
            // there. Because all the places have _a_ value prior to this,
            // there's no problem.
            let hun = [         0,         0, padded[0] ];
            let ten = [         0, padded[0], padded[1] ];
            let one = [ padded[0], padded[1], padded[2] ];

            let [h, t, o] = [
                hun[len],
                ten[len],
                one[len],
            ];

            // If the value is too large, this will error:
            h * 100 + t * 10 + o
        }

        let (major, minor, patch) = (
            ver_str_to_u8(major),
            ver_str_to_u8(minor),
            ver_str_to_u8(patch),
        );

        // The pre-release part seems (relatively) easier to deal with; we just
        // select between `None` and `Some(_)` depending on the length and let
        // the constructor for `Identifier` panic if the version string is too
        // long.
        //
        // Except... this will fail on pre-release version that aren't exactly
        // 4 characters long. To fix this, we need to pad strings that aren't
        // long enough. Luckily, we know how to this now.

        // First catch strings that whose lengths aren't in [0, 4]:
        let pre_release_version_is_too_long = [(); 5];
        pre_release_version_is_too_long[pre.len()];

        // Again, since Cargo checks that only a subset of ASCII is allowed, we
        // can make this a byte string without any fuss:
        let bytes = pre.as_bytes();
        let len = bytes.len();

        // For simplicity we're going to right pad (which we'll then filter out
        // in our `Display` impl and the getter for the pre-release version).
        //
        // TODO: note that we could use this very trick (right padding and
        // doing the below in const constructors for our Identifier and
        // LongIdentifier types, except with NULs instead of spaces...)

        // "abcd" -> "abcd": a(0 -> 0), b(1 -> 1), c(2 -> 2), d(3 -> 3)
        //  "abc" -> "abc ": a(0 -> 0), b(1 -> 1), c(2 -> 2), d(_ -> 3)
        //   "ab" -> "ab  ": a(0 -> 0), b(1 -> 1), c(_ -> 2), d(_ -> 3)
        //    "a" -> "a   ": a(0 -> 0), b(_ -> 1), c(_ -> 2), d(_ -> 3)
        //     "" -> "    ": a(_ -> _), b(_ -> 1), c(_ -> 2), d(_ -> 3)

        // That we also need to handle empty strings throws an additional wrench
        // into the works, but we can solve this with another level of
        // indirection.
        let blank = " ".as_bytes();
        let indirection = [blank, bytes];

        const fn char_at_pos(indir: [&[u8]; 2], len: usize, idx: usize) -> u8 {
            // Length:   0       1       2       3       4
            let a = [(0, 0), (1, 0), (1, 0), (1, 0), (1, 0)];
            let b = [(0, 0), (0, 0), (1, 1), (1, 1), (1, 1)];
            let c = [(0, 0), (0, 0), (0, 0), (1, 2), (1, 2)];
            let d = [(0, 0), (0, 0), (0, 0), (0, 0), (1, 3)];

            let lookup = [a, b, c, d][idx][len];
            let (uno, dos) = lookup;

            indir[uno][dos]
        }

        let padded = [
            char_at_pos(indirection, len, 0),
            char_at_pos(indirection, len, 1),
            char_at_pos(indirection, len, 2),
            char_at_pos(indirection, len, 3),
        ];

        // So now we have a padded string which means we can blindly pass it
        // to the Identifier constructor and be on our merry way.

        let pre = [
            None,
            Some(Identifier::new_that_crashes_on_invalid_inputs(padded)),
        ][[0, 1, 1, 1, 1][len]];

        Self::new(major, minor, patch, pre)
    }
}

#[macro_export]
macro_rules! version_from_crate {
    () => {$crate::control::metadata::Version::from_cargo_env_vars(
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        env!("CARGO_PKG_VERSION_PRE"),
    )};
}

// We re-export the macro from the root to this module. Unfortunately, we can't
// seem to get around the macro also showing up in the crate's root in docs.
pub use crate::version_from_crate;

impl Display for Version {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(pre) = self.get_pre() {
            write!(fmt, "-{}", pre)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Name of the device.
    pub name: Identifier,
    /// Version of the software running on the device.
    /// The exact meaning of this field is up to implementors; by default, our
    /// implementations use the current crate's version.
    pub version: Version,
    /// `TypeId` of the `Control` impl running on the device.
    pub type_id: u64,
    /// Extra functionality supported by the `Control` impl.
    pub capabilities: Capabilities,
    /// The `Identifier`s of any proxies between the device and the `Control`
    /// user.
    pub proxies: [Option<(Identifier, Version)>; 3],
}

impl DeviceInfo {
    const MAX_NUM_PROXIES: usize = 3;

    pub fn new(
        name: Identifier,
        version: Version,
        type_id: TypeId,
        capabilities: Capabilities,
        proxies: [Option<(Identifier, Version)>; Self::MAX_NUM_PROXIES],
    ) -> Self {
        Self {
            name,
            version,
            type_id: type_id.t(),
            capabilities,
            proxies,
        }
    }

    pub fn add_proxy(mut self, proxy: Identifier, version: Version) -> Result<Self, Self> {
        if let Some(idx) = self
            .proxies
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_none())
            .map(|(idx, _)| idx)
            .next()
        {
            self.proxies[idx] = Some((proxy, version));

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

#[rustfmt::skip]
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

#[cfg(test)]
mod version_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_cargo() {
        let vers = version_from_crate!();

        assert_eq!(vers.major.to_string(), env!("CARGO_PKG_VERSION_MAJOR"));
        assert_eq!(vers.minor.to_string(), env!("CARGO_PKG_VERSION_MINOR"));
        assert_eq!(vers.patch.to_string(), env!("CARGO_PKG_VERSION_PATCH"));

        let pre = env!("CARGO_PKG_VERSION_PRE");

        if pre.len() > 0 {
            assert_eq!(vers.get_pre().unwrap(), pre);
        }
    }

    fn single_test(
        major: u16,
        minor: u16,
        patch: u16,
        pre: &'static str,
        expected_pre: Option<&str>,
    ) {
        let ver = Version::from_cargo_env_vars(
            &major.to_string(),
            &minor.to_string(),
            &patch.to_string(),
            pre,
        );

        assert_eq!(ver.major as u16, major);
        assert_eq!(ver.minor as u16, minor);
        assert_eq!(ver.patch as u16, patch);

        assert_eq!(ver.get_pre(), expected_pre);
    }

    #[test]
    fn numbers() {
        single_test(0, 0, 0, "", None);
        single_test(0, 0, 1, "", None);
        single_test(0, 1, 0, "", None);
        single_test(0, 1, 1, "", None);
        single_test(1, 1, 1, "", None);
        single_test(1, 2, 3, "", None);
        single_test(123, 45, 67, "", None);
        single_test(1, 23, 45, "", None);
        single_test(254, 253, 254, "", None);
    }

    #[test]
    #[should_panic]
    fn bad_numbers() {
        single_test(256, 0, 0, "", None);
    }

    #[test]
    fn pre_release_version() {
        single_test(1, 2, 3, "", None);
        single_test(1, 2, 3, "foo", Some("foo"));
        single_test(1, 2, 3, "FOo4", Some("FOo4"));
        single_test(1, 2, 3, "12-3", Some("12-3"));
        single_test(1, 2, 3, "8", Some("8"));
        single_test(1, 2, 3, "a9", Some("a9"));
        single_test(1, 2, 3, "b", Some("b"));
    }

    #[test]
    #[should_panic]
    fn pre_release_version_too_long() {
        single_test(1, 2, 3, "loong", None);
    }
}
