use crate::{core::*};
use super::{Context, RawContext, ContextProperties};

/// A simple RSCL context with a single command queue
#[derive(Clone)]
pub struct SimpleContext {
    ctx: RawContext,
    queue: CommandQueue
}

impl SimpleContext {
    pub fn new (device: &Device, ctx_props: ContextProperties, props: impl Into<QueueProperties>) -> Result<Self> {
        let ctx = RawContext::new(ctx_props, core::slice::from_ref(device))?;
        let queue = CommandQueue::new(props.into(), &ctx, device)?;
        Ok(Self { ctx, queue })
    }
}

impl Context for SimpleContext {
    #[inline(always)]
    fn raw_context (&self) -> &RawContext {
        &self.ctx
    }

    #[inline(always)]
    fn queue_count (&self) -> usize {
        1
    }

    #[inline(always)]
    fn next_queue (&self) -> &CommandQueue {
        &self.queue
    }
}

impl Default for SimpleContext {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Device::first().unwrap(), ContextProperties::default(), QueueProperties::default()).unwrap()
    }
}

unsafe impl Send for SimpleContext {}
unsafe impl Sync for SimpleContext {}