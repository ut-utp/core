//! Traits for [`Input`] and [`Output`] Peripherals that let controllers (like
//! the tui) write and read from them (respectively).
//!
//! [`Input`]: `lc3_traits::peripherals::Input`
//! [`Output`]: `lc3_traits::peripherals::Output`

use lc3_shims::peripherals::{Sink, SourceShim};

use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

/// A trait for [`Input`] Peripherals that lets us, a controller, supply the
/// inputs to the peripheral.
///
/// This is useful for virtual input peripherals like the [`InputShim`] and for
/// other situations where the input peripheral is designed to behave like a
/// serial port or a tty that the controller can use to communicate with the
/// device (i.e. the UART backed `Input`/`Output` peripherals used by default on
/// boards).
///
/// Note that this is already implemented for the default input source
/// ([`SourceShim`]) used by the input shim ([`InputShim`]) but not for all
/// input sources that can be used with [`InputShim`]. If you create your own
/// input source that you implement [`Source`] for, be sure to implement this
/// trait for it as well if you wish to use your input source with a controller
/// application (like the tui).
///
/// Note that this is not a super trait for [`Source`] because it is possible to
/// have [`Input`] implementations that are not the [`InputShim`] communicate
/// with the controller.
///
/// [`Input`]: `lc3_traits::peripherals::Input`
/// [`InputShim`]: `lc3_shims::peripherals::InputShim`
/// [`SourceShim`]: `lc3_shims::peripherals::SourceShim`
/// [`Source`]: `lc3_shims::peripherals::Source`
pub trait InputSink {
    // Note: probably only ASCII for now.
    //
    // Should return `None` on errors/invalid chars.
    fn put_char(&self, c: char) -> Option<()>;
}

/// A trait for [`Output`] Peripherals that lets us, a controller, consume the
/// outputs from the peripheral.
///
/// This is the [`InputSink`] trait's [`Output`] peripheral counterpart.
///
/// Analogous to the [`InputSink`] trait, this is useful for virtual output
/// peripherals like the [`OutputShim`] and for other situations where the
/// output peripheral is designed to behave like a serial port or a tty that the
/// controller can use to communicate with the device (i.e. the UART backed
/// `Input`/`Output` peripherals used by default on boards).
///
/// Note that this is already implemented for the default output source
/// (a `Mutex<Vec<u8>>`) used by the output shim ([`OutputShim`]) as well as
/// every `Mutex` based [`Sink`] whose inner type derefs (mutably) into
/// something that implements [`Read`].
///
/// In the event that you find yourself creating your own output sink that you
/// implement [`Sink`] for (or if your `Mutex` based sink's inner type supports
/// [`Write`] but doesn't `deref_mut` into a [`Read`] implementor), be sure to
/// implement this trait for it as well if you wish to use your input source
/// with a controller application (like the tui).
///
/// Note that this is not a super trait for [`Sink`] because it is possible to
/// have [`Output`] implementations that are not the [`OutputShim`] communicate
/// with the controller (i.e. UART backed peripherals as mentioned above).
///
/// [`Output`]: `lc3_traits::peripherals::Output`
/// [`OutputShim`]: `lc3_shims::peripherals::OutputShim`
/// [`Sink`]: `lc3_shims::peripherals::Sink`
/// [`Read`]: `std::io::Read`
/// [`Write`]: `std::io::Write`
pub trait OutputSource {
    // Note: probably only ASCII for now.
    //
    // Should return `None` when no characters are available.
    fn get_chars(&self) -> Option<String>;
}

// This is fine!
impl InputSink for SourceShim {
    fn put_char(&self, c: char) -> Option<()> {
        self.push(c);
        Some(())
    }
}

// Mirrors the impl on `SourceShim`.
impl InputSink for Arc<SourceShim> {
    fn put_char(&self, c: char) -> Option<()> {
        self.push(c);
        Some(())
    }
}

// TODO: fix this!
// // Mirrors the blanket impl that `Sink` has but also requires `Read` support so
// // that we can actually implement OutputSource.
// impl<W: Write> OutputSource for Mutex<W>
// where
//     Mutex<W>: Sink, // This is really guaranteed.
//     W: Deref,
//     // W: DerefMut,
//     for<'r> &'r <W as Deref>::Target: Read,
// {
//     fn get_chars(&self) -> Option<String> {
//         let mut s = String::new();
//         let source = self.lock().unwrap();

//         // This will just not pull characters into the output string if we
//         // encounter non-utf8 characters (see the docs for
//         // `Read::read_to_string`).
//         // TODO: maybe handle non-utf8 chars differently.
//         // let mut r: &<W as Deref>::Target = source.deref();
//         <W as Deref>::deref(&source)
//             .read_to_string(&mut s)
//             .ok()
//             .filter(|n| *n > 0)
//             .map(|_| s)
//     }
// }

// Note that `Vec` actually only supports `Read` by way of `Deref`-ing into a
// `u8` slice; this is no good for us becuase it means that
// `Read::read_to_string` doesn't actually remove the characters from the `Vec`.
// So, we have the special impl for Vec below:
impl OutputSource for Mutex<Vec<u8>> {
    fn get_chars(&self) -> Option<String> {
        let mut v = self.lock().unwrap();

        // Unlike the impl above, this will throw away any non-Unicode
        // characters along with everything else that's currently in the `Vec`.
        // TODO: maybe handle non-utf8 characters differently or at least be
        // consistent with the impl above.
        let s = v.drain(..).collect();

        String::from_utf8(s).ok().filter(|s| s.len() > 0)

        // let s: String = v.drain(..)
        //     .collect::<Option<String>>()
        //     .ok()
        //     .filter(|n| *n > 0)
        //     .map(|_| s)
    }
}

// Mirrors the blanket impl that `Sink` has.
impl<O: OutputSource> OutputSource for Arc<O> {
    fn get_chars(&self) -> Option<String> {
        O::get_chars(self)
    }
}
