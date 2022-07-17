use std::{ops::Deref, sync::atomic::AtomicUsize};
use rscl_proc::docfg;

use crate::prelude::{CommandQueue, Device, Result, QueueProperties, device::PartitionProperty};
use super::{RawContext, ContextProperties, Context};

pub struct CyclicContext {
    ctx: RawContext,
    queues: Box<[CommandQueue]>,
    idx: AtomicUsize
}

impl CyclicContext {
    #[inline]
    pub fn new (devices: &[Device], ctx_props: ContextProperties, props: impl Into<QueueProperties>) -> Result<Self> {
        assert!(devices.len() > 0);

        let props = props.into();
        let ctx = RawContext::new(ctx_props, devices)?;
        let mut queues = Box::new_uninit_slice(devices.len());

        for (i, device) in devices.into_iter().enumerate() {
            #[cfg(feature = "cl2_1")]
            println!("{:?}", device.max_num_sub_groups());
            let queue = CommandQueue::new(&ctx, props, device)?;
            queues[i].write(queue);
        }

        let queues = unsafe { queues.assume_init() };
        Ok(Self { ctx, queues, idx: AtomicUsize::default() })
    }

    #[docfg(feature = "cl1_2")]
    #[inline]
    pub fn from_split (device: &Device, sub_prop: PartitionProperty, ctx_props: ContextProperties, props: impl Into<QueueProperties>) -> Result<Self> {
        let devices = device.create_sub_devices(sub_prop)?;
        Self::new(&devices, ctx_props, props)
    }
}

impl Context for CyclicContext {
    #[inline(always)]
    fn queues (&self) -> &[CommandQueue] {
        &self.queues
    }

    #[inline]
    fn next_queue (&self) -> &CommandQueue {
        let idx = self.idx.fetch_update(std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire, |prev| {
            let next = prev + 1;
            if next == self.queues.len() {
                return Some(0)
            }

            Some(next)
        }).unwrap();

        &self.queues[idx]
    }
}

impl Deref for CyclicContext {
    type Target = RawContext;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}