flat_mod!(global, single);
use opencl_sys::{cl_context, cl_command_queue};

/// An object that can be used as a RSCL context, with a similar syntax to Rust allocators.
/// 
/// RSCL contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows RSCL contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context {
    fn context_id (&self) -> cl_context;
    fn queue_count (&self) -> usize;
    fn next_queue (&self) -> cl_command_queue;
}