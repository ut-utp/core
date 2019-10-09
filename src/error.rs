use super::Word;
use super::peripherals::gpio::{GpioReadError, GpioWriteError, GpioReadErrors, GpioWriteErrors, GpioMiscError/* GpioInterruptRegisterError */};


// Lots of open questions here:
//  - should this be implementation defined?
//    + we do get some nice benefits from sticking this in the Control trait
//  - what error variants?
//  - so interpreters get to decide how to communicate errors to the LC-3 programs?
//    + and really that just means what, picking default values to return instead of, for example, crashing on a InvalidGpioRead?
//    + but there are probably some errors we _do_ want to actually fire exceptions on (note: we'll need new exceptions!)
//    + I'm warming to this idea, actually. The underlying infrastructure (peripherals, control) agree on a set of errors; how those
//      errors make their way into LC-3 land is up to the interpreter. It's literally a matter of mapping these Errors into whatever.
pub enum Error {
    InvalidGpioWrite(GpioWriteError),
    InvalidGpioWrites(GpioWriteErrors),
    InvalidGpioRead(GpioReadError),
    InvalidGpioReads(GpioReadErrors),
    GpioMiscError(GpioMiscError), // Unclear if we want to expose these kind of errors in the Control interface or just make the interpreter deal with them (probably expose...) (TODO)
    // InvalidGpioInterruptRegistration(GpioInterruptRegisterError),
    ///// TODO: finish
}

// TODO: automate away with a proc macro (this is a common enough pattern...)
// or at least a macro_rules macro

// impl From<GpioWriteError> for Error {
//     fn from(e: GpioWriteError) -> Error {
//         Error::InvalidGpioWrite(e)
//     }
// }

// TODO: also implement the Try trait behind a feature gate?
// Actually, no; it doesn't really help here.

macro_rules! err {
    ($type:ty, $variant:path) => {
        impl From<$type> for Error {
            fn from(e: $type) -> Self {
                $variant(e)
            }
        }
    };
}

err!(GpioWriteError, Error::InvalidGpioWrite);
// TODO: finish

/// Just some musings; if we go with something like this it won't live here.
///
/// TBD on whether this is impl-defined.
/// Another thing to consider is that we may want different modes? Permissive and strict or something. Or maybe not.
pub enum ErrorHandlingStrategy {
    DefaultValue(Word),
    Silent,
    FireException { interrupt_vector_table_number: u8, payload: Option<Word> },
}

impl From<Error> for ErrorHandlingStrategy {
    fn from(err: Error) -> Self {
        use Error::*;
        use ErrorHandlingStrategy::*;

        match err {
            InvalidGpioWrite(_) => Silent,
            InvalidGpioRead(_) => DefaultValue(0u16),
            InvalidGpioWrites(_) => Silent,
            InvalidGpioReads(err) => {
                unimplemented!()
                // TODO: set all the mismatched bits to 0, etc.
            },
            GpioMiscError(_) => Silent,
        }
    }
}
