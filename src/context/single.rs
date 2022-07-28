use std::ops::Deref;
use blaze_proc::docfg;

use crate::{core::*};
use super::{Context, RawContext, ContextProperties};

/// A simple RSCL context with a single command queue
#[derive(Clone)]
pub struct SimpleContext {
    ctx: RawContext,
    queue: RawCommandQueue
}

impl SimpleContext {
    pub fn new (device: &RawDevice, ctx_props: ContextProperties, props: impl Into<QueueProperties>) -> Result<Self> {
        let ctx = RawContext::new(ctx_props, core::slice::from_ref(device))?;
        let queue = RawCommandQueue::new(&ctx, props.into(), device)?;
        Ok(Self { ctx, queue })
    }

    #[docfg(feature = "cl3")]
    pub fn with_logger (device: &RawDevice, ctx_props: ContextProperties, props: impl Into<QueueProperties>, loger: impl 'static + Fn(&str) + Send) -> Result<Self> {
        let ctx = RawContext::with_logger(ctx_props, core::slice::from_ref(device), loger)?;
        let queue = RawCommandQueue::new(&ctx, props.into(), device)?;
        Ok(Self { ctx, queue })
    }

    #[inline(always)]
    pub fn default() -> Result<Self> {
        let device = RawDevice::first().ok_or(ErrorType::InvalidDevice)?;

        cfg_if::cfg_if! {
            if #[cfg(all(debug_assertions, feature = "cl3"))] {
                Self::with_logger(device, ContextProperties::default(), QueueProperties::default(), |x| println!("{x}"))
            } else {
                Self::new(device, ContextProperties::default(), QueueProperties::default())
            }
        }
    }
}

impl Context for SimpleContext {
    #[inline(always)]
    fn queues (&self) -> &[RawCommandQueue] {
        core::slice::from_ref(&self.queue)
    }

    #[inline(always)]
    fn next_queue (&self) -> &RawCommandQueue {
        &self.queue
    }
}

impl Deref for SimpleContext {
    type Target = RawContext;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}