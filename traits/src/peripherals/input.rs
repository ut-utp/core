//! [`Input` device trait](Input) and related things.

use crate::peripheral_trait;

peripheral_trait! {input,
pub trait Input: Default {}
}
