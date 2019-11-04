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
/// 3) What if we went further?
pub fn __() -> () {}
