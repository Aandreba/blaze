use std::ops::Deref;
use super::{Context, RawContext, CommandQueue};

extern "Rust" {
    fn __blaze__global__as_raw () -> &'static RawContext;
    fn __blaze__global__queues () -> &'static [CommandQueue];
    fn __blaze__global__next_queue () -> &'static CommandQueue;
}

#[doc = include_str!("../../docs/src/context/global.md")]
#[derive(Copy, Clone, Default, Debug)]
pub struct Global;

impl Context for Global {
    #[inline(always)]
    fn next_queue (&self) -> &CommandQueue {
        unsafe { __blaze__global__next_queue() } 
    }

    #[inline(always)]
    fn as_raw (&self) -> &RawContext {
        unsafe { __blaze__global__as_raw() } 
    }

    #[inline(always)]
    fn queues (&self) -> &[CommandQueue] {
        unsafe { __blaze__global__queues() }
    }
}

impl Deref for Global {
    type Target = RawContext;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_raw()
    }
}