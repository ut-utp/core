use super::peripherals::gpio::{
    GpioMiscError, /* GpioInterruptRegisterError */
    GpioReadError, GpioReadErrors, GpioWriteError, GpioWriteErrors,
};
use super::peripherals::adc::{AdcReadError, AdcReadErrors, AdcMiscError};
use super::peripherals::input::InputError;
use super::peripherals::output::OutputError;
use lc3_isa::Word;

use core::fmt::Display;

use serde::{Serialize, Deserialize};

// Lots of open questions here:
//  - should this be implementation defined?
//    + we do get some nice benefits from sticking this in the Control trait
//  - what error variants?
//  - so interpreters get to decide how to communicate errors to the LC-3 programs?
//    + and really that just means what, picking default values to return instead of, for example, crashing on a InvalidGpioRead?
//    + but there are probably some errors we _do_ want to actually fire exceptions on (note: we'll need new exceptions!)
//    + I'm warming to this idea, actually. The underlying infrastructure (peripherals, control) agree on a set of errors; how those
//      errors make their way into LC-3 land is up to the interpreter. It's literally a matter of mapping these Errors into whatever.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Error {
    InvalidGpioWrite(GpioWriteError),
    InvalidGpioWrites(GpioWriteErrors),
    InvalidGpioRead(GpioReadError),
    InvalidGpioReads(GpioReadErrors),
    GpioMiscError(GpioMiscError), // Unclear if we want to expose these kind of errors in the Control interface or just make the interpreter deal with them (probably expose...) (TODO)
                                  // InvalidGpioInterruptRegistration(GpioInterruptRegisterError),
    InvalidAdcRead(AdcReadError),
    InvalidAdcReads(AdcReadErrors),
    AdcMiscError(AdcMiscError),

    InputError(InputError),
    OutputError(OutputError),

    SystemStackOverflow,
    ///// TODO: finish
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidGpioWrite(err) => {
                return write!(f, "Attempted to write to {} when in {} mode", (err.0).0, (err.0).1);
            }
            Error::InvalidGpioWrites(_) => {
                unimplemented!();       // TODO
            }
            Error::InvalidGpioRead(err) => {
                return write!(f, "Attempted to read from {} when in {} mode", (err.0).0, (err.0).1);
            }
            Error::InvalidGpioReads(_) => {
                unimplemented!();       // TODO
            }
            Error::GpioMiscError(_) => {
                unimplemented!();       // TODO
            }
            Error::InvalidAdcRead(err) => {
                return write!(f, "Attempted to read from {} when in {} mode", (err.0).0, (err.0).1);
            }
            Error::InvalidAdcReads(_) => {
                unimplemented!();       // TODO
            }
            Error::AdcMiscError(_) => {
                unimplemented!();       // TODO
            }
            Error::InputError(_) => {
                unimplemented!();       // TODO
            }
            Error::OutputError(_) => {
                unimplemented!();       // TODO
            }
            Error::SystemStackOverflow => {
                return write!(f, "Overflowed system stack");
            }
        }
    }
}

using_std! { impl std::error::Error for Error { } }

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
err!(GpioWriteErrors, Error::InvalidGpioWrites);
err!(GpioReadError, Error::InvalidGpioRead);
err!(GpioReadErrors, Error::InvalidGpioReads);
err!(GpioMiscError, Error::GpioMiscError);
err!(AdcReadError, Error::InvalidAdcRead);
err!(AdcReadErrors, Error::InvalidAdcReads);
err!(AdcMiscError, Error::AdcMiscError);
err!(InputError, Error::InputError);
err!(OutputError, Error::OutputError);
// TODO: finish

/// Just some musings; if we go with something like this it won't live here.
///
/// TBD on whether this is impl-defined.
/// Another thing to consider is that we may want different modes? Permissive and strict or something. Or maybe not.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    DefaultValue(Word),
    Silent,
    FireException {
        interrupt_vector_table_number: u8,
        payload: Option<Word>,
    },
}

impl From<Error> for ErrorHandlingStrategy {
    fn from(err: Error) -> Self {
        use Error::*;
        use ErrorHandlingStrategy::*;

        match err {
            InvalidGpioWrite(_) => Silent,
            InvalidGpioRead(_) => DefaultValue(0u16),
            InvalidGpioWrites(_) => Silent,
            InvalidGpioReads(_err) => {
                unimplemented!()
                // TODO: set all the mismatched bits to 0, etc.
            }
            GpioMiscError(_) => Silent,
            InvalidAdcRead(_) => DefaultValue(0u16),
            InvalidAdcReads(_) => {
                unimplemented!()
            }
            AdcMiscError(_) => Silent,
            InputError(_) => Silent,        // TODO: what to actually do here?
            OutputError(_) => Silent,       // TODO: and here?
            SystemStackOverflow => Silent,
        }
    }
}
