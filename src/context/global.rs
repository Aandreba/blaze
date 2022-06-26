use opencl_sys::{cl_context, cl_command_queue};
use super::Context;

extern "Rust" {
    fn __rscl__global__context_id () -> cl_context;
    fn __rscl__global__queue_count () -> usize;
    fn __rscl__global__next_queue () -> cl_command_queue;
}

/// RSCL's global context
#[derive(Copy, Clone, Default, Debug)]
pub struct Global;

impl Context for Global {
    #[inline(always)]
    fn context_id (&self) -> cl_context {
        unsafe { __rscl__global__context_id() } 
    }

    #[inline(always)]
    fn queue_count (&self) -> usize {
        unsafe { __rscl__global__queue_count() } 
    }

    #[inline(always)]
    fn next_queue (&self) -> cl_command_queue {
        unsafe { __rscl__global__next_queue() } 
    }
}