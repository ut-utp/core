//! Extensions to the [`Control`] trait (just sugar).
//!
//! [`Control`]: super::Control

use super::Control;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DepthBreakpoint {
    StepOut,
    StepIn,
    StepOver,
}

pub trait StepControl: Control {
    fn set_depth_breakpoint(&mut self, bp: DepthBreakpoint) -> Result<(), ()> {
        let curr = self.get_depth()?;

        #[forbid(unreachable_patterns)]
        let range = match bp {
            DepthBreakpoint::StepOut => (..curr).into(),
            DepthBreakpoint::StepIn => (curr..).into(),
            DepthBreakpoint::StepOver => (..=curr).into(),
        };

        self.set_depth_condition(range).map(|_| ())
    }

    fn step_out(&mut self) -> Result<(), ()> {
        self.set_depth_breakpoint(DepthBreakpoint::StepOut)
    }

    fn step_in(&mut self) -> Result<(), ()> {
        self.set_depth_breakpoint(DepthBreakpoint::StepIn)
    }

    fn step_over(&mut self) -> Result<(), ()> {
        self.set_depth_breakpoint(DepthBreakpoint::StepOver)
    }
}

impl<C: Control + ?Sized> StepControl for C { }
