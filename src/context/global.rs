use crate::core::CommandQueue;
use super::{Context, RawContext};

extern "Rust" {
    fn __rscl__global__raw_context () -> &'static RawContext;
    fn __rscl__global__queue_count () -> usize;
    fn __rscl__global__next_queue () -> &'static CommandQueue;
}

/// RSCL's global context
#[derive(Copy, Clone, Default, Debug)]
pub struct Global;

impl Context for Global {
    #[inline(always)]
    fn raw_context (&self) -> &RawContext {
        unsafe { __rscl__global__raw_context () } 
    }

    #[inline(always)]
    fn queue_count (&self) -> usize {
        unsafe { __rscl__global__queue_count() } 
    }

    #[inline(always)]
    fn next_queue (&self) -> &CommandQueue {
        unsafe { __rscl__global__next_queue() } 
    }
}