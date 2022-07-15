use std::ops::Deref;
use crate::core::CommandQueue;
use super::{Context, RawContext};

extern "Rust" {
    fn __rscl__global__as_raw () -> &'static RawContext;
    fn __rscl__global__queues () -> &'static [CommandQueue];
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

    #[inline(always)]
    fn as_raw (&self) -> &RawContext {
        unsafe { __rscl__global__as_raw() } 
    }

    #[inline(always)]
    fn queues (&self) -> &[CommandQueue] {
        unsafe { __rscl__global__queues() }
    }
}

impl Deref for Global {
    type Target = RawContext;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_raw()
    }
}