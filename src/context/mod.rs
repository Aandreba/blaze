flat_mod!(raw,flags, global, single);
use std::ops::Deref;

use crate::core::CommandQueue;

/// An object that can be used as a RSCL context, with a similar syntax to Rust allocators.\
/// RSCL contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows RSCL contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context: Deref<Target = RawContext> {
    fn next_queue (&self) -> &CommandQueue;

    #[inline(always)]
    fn raw_context (&self) -> &RawContext {
        self
    }
}