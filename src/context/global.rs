use std::ops::Deref;
use crate::core::CommandQueue;
use super::{Context, RawContext};

extern "Rust" {
    fn __rscl__global__raw_context () -> &'static RawContext;
    fn __rscl__global__next_queue () -> &'static CommandQueue;
}

/// RSCL's global context
#[derive(Copy, Clone, Default, Debug)]
pub struct Global;

impl Context for Global {
    #[inline(always)]
    fn next_queue (&self) -> &CommandQueue {
        unsafe { __rscl__global__next_queue() } 
    }
}

impl Deref for Global {
    type Target = RawContext;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { __rscl__global__raw_context () } 
    }
}