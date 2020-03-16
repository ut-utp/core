//! TODO!

use super::{BlackBox, Init};
use crate::shim_support::Shims;

#[derive(Debug)]
pub struct BoardDevice {}

// impl<'s> Init<'s> for BoardDevice {
//     type Config = ();

//     type ControlImpl = !;
//     type Input = !;
//     type Output = !;

//     fn init(
//         b: &'s mut BlackBox,
//     ) -> (
//         &'s mut Self::ControlImpl,
//         Option<Shims<'static>>,
//         Option<&'s Self::Input>,
//         Option<&'s Self::Output>,
//     ) {
//         unimplemented!();
//     }
// }
