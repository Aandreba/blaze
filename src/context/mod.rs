flat_mod!(raw,flags, global, single);
use crate::core::CommandQueue;

/// An object that can be used as a RSCL context, with a similar syntax to Rust allocators.\
/// RSCL contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows RSCL contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context {
    fn raw_context (&self) -> &RawContext;
    fn queue_count (&self) -> usize;
    fn next_queue (&self) -> &CommandQueue;
}