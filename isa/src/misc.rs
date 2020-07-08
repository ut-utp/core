//! Miscellaneous odds and ends that are loosely related to the LC-3 ISA.
//!
//! (TODO!)

/// Formally, the input and output devices that are part of the LC-3 only
/// support ASCII characters. We'd like to support more than just ASCII; ideally
/// Unicode text in general but therein lies a problem.
///
/// The LC-3 has a [`Word`](crate::Word) size of 16 bits. UTF-8 (seemingly the
/// de facto Unicode encoding these days) encoded characters can occupy a
/// variable number of bytes (between 1 and 4, inclusive). This is problematic
/// for our LC-3 because:
///   - we're Word (2 bytes) addressable, not byte addressable
///   - the interface for character input and output isn't suited to variable
///     lengths of data (currently users stick their character in R0 or in the
///     lower 8 bits of DSR; users don't and _can't_ specify how many bytes
///     their character is supposed to occupy).
///
/// We have some options:
///
/// 1) Don't try to support any kind of Unicode characters. Stick with ASCII.
///
///    I mean, okay. 🙄 No breaking changes and the least amount of effort and
///    all that but let's see what our other options are.
///
/// 2) Take advantage of the fact that we're a 16 bit word machine and use the
///    other 8 bits that are currently going completely unused. Modify the KBDR
///    and DDR (input and output data registers) to take 16 bit values and
///    modify the GETC, OUT, PUTS, and IN trap routines to support the 16 bit
///    values. Ditch PUTSP.
///
///    The question then becomes what encoding to use? Since we're limiting
///    ourselves to 2 bytes we're not going to be able to support all of Unicode
///    which is maybe an acceptable compromise. But we're definitely going to
///    still want to support ASCII so if we use UTF-8 we're going to have to
///    handle 1 byte characters as well as two.
///
///    Let's illustrate why this is even a problem. As mentioned, UTF-8
///    character encodings occupy a variable number of bytes (depending on the
///    character you're trying to represent). UTF-8 accomplishes this by looking
///    at leading bits in the first byte:
///
///    ```text
///     1 byte:  0b0xxxxxxx
///     2 bytes: 0b110xxxxx_10xxxxxx
///     3 bytes: 0b1110xxxx_10xxxxxx_10xxxxxx
///     4 bytes: 0b11110xxx_10xxxxxx_10xxxxxx_10xxxxxx
///    ```
///
///    This is pretty cool! It means that you can uniquely identify every byte
///    in a Unicode character as a starting byte or a continuation of a
///    character. It also means that you can still parse characters that are
///    being streamed to you byte by byte so long as big endian is being used.
///
///    More importantly, it's backwards compatible with ASCII; all ASCII
///    characters are single byte Unicode characters.
///
///    All this is pretty great, but there's one annoying caveat for us.
///
///    Because we don't have a way, with the described interface, to specially
///    denote that we've only got one 1 byte character. We're always passing
///    along 2 bytes of data; consider what happens when we pass an ASCII
///    character along, 'A' for example:
///
///    ```rust,should_panic
///    # use lc3_isa::Word;
///    let c: Word = 0x41;
///    let buf: [u8; 2] = c.to_be_bytes();
///
///    let s = core::str::from_utf8(&buf).unwrap();
///    assert_eq!("A", s);
///    ```
///
///    The above fails! It complains with:
///
///    ```text
///     stderr:
///     thread 'main' panicked at 'assertion failed: `(left == right)`
///       left: `"A"`,
///      right: `"\u{0}A"`
///    ```
///
///    So what's going on? As mentioned, the issue is that we're always sending
///    two bytes along to be processed as characters.
///
///    `(0x41).to_be_bytes()` gives us back `[0x00, 0x41]`; `0x00` gets processed
///    as it's own character and produces the `"\u{0}"` we see above.
///
///    This is definitely something we could work around. We could add some kind
///    of special termination character that Unicode reserves that denotes that
///    we're sending a one byte character. Or, since we know that we're only
///    looking for one character, we could just start from the back and stop once
///    we've got our character.
///
///    Here's an implementation of the latter:
///
///    ```rust
///    # use lc3_isa::Word;
///    fn encode<'a>(string: &'a str) -> impl Iterator<Item = Word> + 'a {
///        string.chars().map(|c| {
///            match c.len_utf8() {
///                3..=4 => panic!("Can't represent `{}` in <=2 bytes!", c),
///                2 => {
///                    let mut buf: [u8; 2] = [0, 0];
///                    c.encode_utf8(&mut buf);
///                    u16::from_be_bytes(buf)
///                },
///                1 => {
///                    let mut buf: [u8; 1] = [0];
///                    c.encode_utf8(&mut buf);
///                    u16::from_le_bytes([buf[0], 0])
///                },
///                _ => unreachable!(),
///            }
///        })
///    }
///
///    fn decode(iter: impl Iterator<Item = Word>) -> String {
///        let mut string = String::new();
///
///        iter.for_each(|c| {
///            let b = c.to_be_bytes();
///            let s = std::str::from_utf8(&b).unwrap();
///
///            string.push(s.chars().nth_back(0).unwrap());
///        });
///
///        string
///    }
///
///    let orig = "Ԋəllo World¡";
///    assert_eq!(orig, decode(encode(orig)));
///    ```
///
///    As expected, this panics on things it can't represent in two bytes:
///
///    ```rust,should_panic
///    # use lc3_isa::Word;
///    #
///    # fn encode<'a>(string: &'a str) -> impl Iterator<Item = Word> + 'a {
///    #     string.chars().map(|c| {
///    #         match c.len_utf8() {
///    #             3..=4 => panic!("Can't represent `{}` in <=2 bytes!", c),
///    #             2 => {
///    #                 let mut buf: [u8; 2] = [0, 0];
///    #                 c.encode_utf8(&mut buf);
///    #                 u16::from_be_bytes(buf)
///    #             },
///    #             1 => {
///    #                 let mut buf: [u8; 1] = [0];
///    #                 c.encode_utf8(&mut buf);
///    #                 u16::from_le_bytes([buf[0], 0])
///    #             },
///    #             _ => unreachable!(),
///    #         }
///    #     })
///    # }
///    #
///    # fn decode(iter: impl Iterator<Item = Word>) -> String {
///    #     let mut string = String::new();
///    #
///    #     iter.for_each(|c| {
///    #         let b = c.to_be_bytes();
///    #         let s = std::str::from_utf8(&b).unwrap();
///    #
///    #         string.push(s.chars().nth_back(0).unwrap());
///    #     });
///    #
///    #     string
///    # }
///    #
///    assert_eq!("🎉", decode(encode("🎉")));
///    ```
///
///    So clearly, this is an _option_, but it's a bit hacky. Luckily, there's a
///    better way.
///
///    As mentioned, UTF-8 is but one Unicode encoding. Another is UTF-16:
///    similar to UTF-8 it's a variable length encoding but with 16 bit units
///    instead of bytes. Encoded characters can occupy two or four bytes.
///
///    This solves the problem we tried to solve above. To abide by our
///    DDR/KBDR/R0 size limitations we'd have to limit ourselves to the
///    characters that can be encoded in two bytes. It turns out this is actually
///    an (old) encoding of it's own: UCS-2. It's since fallen out of use since
///    it can't represent all the Unicode codepoints in existence today but if
///    we're going to limit ourselves to two bytes anyways this isn't a concern.
///
///    So, to recap, if we're okay with slightly tweaking the TRAPs and using
///    some unused bits in registers (but not modifying their sizes) we can gain
///    access to some of the Unicode codepoints *and* retain backwards
///    compatibility with the existing calling conventions and such.
///
///    Additionally, if we go with the hacky UTF-8 based scheme in the code
///    above we can make PUTSP (and it's assembler directive counterpart) Unicode
///    aware (unpack two 1 byte characters stored in a word; send 2 byte chars in
///    one go; probably don't splice 2 byte chars across words).
///
///    From a backwards compatibility standpoint this is basically perfect. As an
///    added bonus no additional complexity is foist upon users unless they want
///    the additional functionality.
///
///    The only real downside is that this doesn't provide full Unicode support.
///    Supporting other languages is really the point of this (the emojis are
///    nice too though) and, for example, CJK codepoints require 3 bytes in
///    UTF-8. So:
///
/// 3) What if we went further? If backwards compatibility weren't a concern _at
///    all_, what might we do?
///
///    UCS-4 or UTF-16 characters. We could do UTF-8 but dealing with characters
///    spliced across word boundaries seems extremely messy and not worth the
///    space savings.
///
///    Length prefixed strings, Java style. The first word of a string has the
///    number of *words* (N) that the string occupies *not including that
///    starting word*. The following N words contain the strings characters, in
///    order. Note: If we were to go with UCS-4 this would be sufficient to
///    determine the number of characters in the string; with UTF-16 it is not.
///
///    KBDR and DDR are now 2 words each. Writes to the low (least significant)
///    word cause the display peripheral to actually process the data in the low
///    and high words. If the high word was not set since the words were last
///    processed, a single word UTF-16 character is assumed. The keyboard
///    peripheral will grow a bit in the status register indicating whether the
///    current character is one or two UTF-16 words. Note: If we were to go with
///    UCS-4 the DDR would read both bytes always and there would be no extra
///    bit in the KBSR.
///
///    The single character TRAPs now take pointers:
///      - GETC: Pointer to an existing string in R0.
///        + Conveniently, mem\[R0\] == 0 is interpreted as an existing string
///          that is empty.
///        + This TRAP will now append the character that's read in onto the end
///          of the string.
///        + R0 remains as is, R1 will contain the address of the appended
///          character.
///        + Sidenote: I think it makes sense for this TRAP to now take a
///          pointer to a string instead of a pointer of a memory location to
///          place a character so that the callee doesn't have to figure out how
///          many words the appended character occupies (in the case of UTF-16).
///          * It's also just nicer.
///      - OUT: Pointer to the starting word of a character in R0.
///        + Starting word assumes UTF-16; if UCS-4 then just the top word.
///        + In the case of UTF-16, this TRAP would figure out if the char is
///          one or two words.
///      - PUTS: Pointer to an existing string in R0.
///        + R1 will contain the number of characters printed out.
///      - IN: Pointer to an existing string in R0.
///        + Same behavior as `GETC` when passed in 0.
///        + R1 will contain the number of characters appended to the string.
///      - PUTSP: Gone.
///        + There isn't a more packed representation with UTF-16 or UCS-4.
///
///    The preprocessor string directives get simpler:
///      - .FILL: Encodes a character into either one or two words (if UTF-16).
///        + Actually this is a little more complicated than before.
///      - .STRINGZ: Encodes a string into the representation detailed above.
///        + Okay, this is more also more complicated, but no NULL termination
///          or anything.
///      - <packed string directive>: As with PUTSP, gone!
///
///    And I think that covers it!
///
///    The upside is full Unicode compatibility! And a more sane string type.
///    And nicer TRAPs.
///
///    The downsides, of course, are many:
///      - Absolutely no backwards compatibility.
///      - Would *absolutely* break existing code.
///        + the TRAPs are entirely different not to mention strings now occupy
///          a different amount of space (possible) since there is no packed
///          representation
///      - It's not zero cost.
///        + This a downside of UTF-16 and UCS-4; if all I care about is ASCII
///          I'm still paying a size penalty.
///        + There's also a performance penalty (more checks, though less so for
///          UCS-4)! We don't really care about this though.
///      - This is potentially more complexity than we'd want to expose on a
///        pedagogic system. We routinely choose to hide messy real world
///        complexities so that things are easier to teach and understand and
///        Unicode probably qualifies as something we should hide.
///        + I actually disagree with this:
///          * I think the above model isn't all that complicated.
///          * I think you only need a very surface level understanding of
///            Unicode to use the above.
///          * More importantly, I think basic Unicode is essentially *required*
///            today; perpetuating the idea that you can do with ASCII in the
///            real world probably borders on irresponsible.
///        + That said, not being backwards compatible with ASCII *is* a deal
///          breaker for something in this category (an educational tool).
///
/// Fwiw, [this LC-3 assembler crate](https://github.com/cr0sh/lc3asm) seems to
/// have support for UTF-8 string literals iiuc; I'm not sure how (if at all)
/// they deal with UTF-8 strings/characters in the userspace.
pub fn __() -> () {}

pub mod util {
    /// Associated types and other weird bits for the LC-3 ISA.
    use crate::{Addr, Word, ADDR_SPACE_SIZE_IN_WORDS};

    use core::ops::{Deref, DerefMut};

    // TODO: on `std` impl `MemoryDump` from `io::Read`?

    // Newtype
    #[derive(Clone)] // TODO: impl Debug + PartialEq/Eq + Ser/De + Hash
    pub struct MemoryDump(pub [Word; ADDR_SPACE_SIZE_IN_WORDS]);
    impl Deref for MemoryDump {
        type Target = [Word; ADDR_SPACE_SIZE_IN_WORDS];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for MemoryDump {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<[Word; ADDR_SPACE_SIZE_IN_WORDS]> for MemoryDump {
        fn from(memory: [Word; ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
            Self(memory)
        }
    }

    impl MemoryDump {
        pub fn blank() -> Self {
            [0; ADDR_SPACE_SIZE_IN_WORDS].into()
        }

        pub fn layer_loadable<L: LoadableIterator>(&mut self, loadable: L) -> &mut Self {
            for (addr, word) in loadable {
                self[addr as usize] = word;
            }

            self
        }

        // TODO: provide a trait for this too
        // TODO: does it make sense to impl FromIterator for any of these types?
        pub fn layer_iterator<I: Iterator<Item = (Addr, Word)>>(&mut self, iter: I) -> &mut Self {
            for (addr, word) in iter {
                self[addr as usize] = word;
            }

            self
        }
    }

    type AssembledProgramInner = [(Word, bool); ADDR_SPACE_SIZE_IN_WORDS];

    impl From<AssembledProgram> for MemoryDump {
        fn from(memory: AssembledProgram) -> Self {
            let mut mem: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0; ADDR_SPACE_SIZE_IN_WORDS];

            memory
                .iter()
                .enumerate()
                .for_each(|(idx, (w, _))| mem[idx] = *w);

            Self(mem)
        }
    }

    impl From<AssembledProgramInner> for MemoryDump {
        fn from(memory: AssembledProgramInner) -> Self {
            Into::<AssembledProgram>::into(memory).into()
        }
    }

    // Newtype
    #[derive(Clone)] // TODO: impl Debug + PartialEq/Eq + Ser/De + Hash
    pub struct AssembledProgram(pub [(Word, bool); ADDR_SPACE_SIZE_IN_WORDS]);
    impl AssembledProgram {
        pub const fn new(mem: [(Word, bool); ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
            Self(mem)
        }
    }

    impl Deref for AssembledProgram {
        type Target = AssembledProgramInner;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for AssembledProgram {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<AssembledProgramInner> for AssembledProgram {
        fn from(prog: AssembledProgramInner) -> Self {
            Self(prog)
        }
    }

    // pub trait LoadableIterator<'a>: IntoIterator<Item = &'a (Addr, Word)> + Sized {
    pub trait LoadableIterator: IntoIterator<Item = (Addr, Word)> + Sized {
        fn to_memory_dump(self) -> MemoryDump {
            let mut mem: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0; ADDR_SPACE_SIZE_IN_WORDS];

            self.into_iter()
                .for_each(|(addr, word)| mem[addr as usize] = word);

            mem.into()
        }
    }

    impl<I: IntoIterator<Item = (Addr, Word)>> LoadableIterator for I {}

    use core::{
        iter::{Enumerate, Filter, Map},
        slice::Iter,
    };

    impl<'a> IntoIterator for &'a MemoryDump {
        type Item = (Addr, Word);
        // type IntoIter = MemoryDumpLoadableIterator<'a>;
        type IntoIter = Map<Enumerate<Iter<'a, Word>>, &'a dyn Fn((usize, &Word)) -> (Addr, Word)>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
                .enumerate()
                .map(&|(idx, word)| (idx as Addr, *word))
        }
    }

    impl<'a> IntoIterator for &'a AssembledProgram {
        type Item = (Addr, Word);
        type IntoIter = Map<
            Filter<Enumerate<Iter<'a, (Word, bool)>>, &'a dyn Fn(&(usize, &(u16, bool))) -> bool>,
            &'a dyn Fn((usize, &(Word, bool))) -> (Addr, Word),
        >;

        #[allow(trivial_casts)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
                .enumerate()
                .filter(
                    (&|(_, (_, set)): &(usize, &(Word, bool))| *set)
                        as &(dyn Fn(&(usize, &(u16, bool))) -> bool),
                ) // This cast is marked as trivial but it's not, apparently
                .map(&|(idx, (word, _)): (usize, &(Word, bool))| (idx as Addr, *word))
        }
    }
}
