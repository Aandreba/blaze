flat_mod!(eventual);

use std::sync::{Arc, atomic::AtomicUsize};
use crate::prelude::{RawCommandQueue, WaitList, Result, RawBuffer};

pub struct CommandQueue {
    inner: RawCommandQueue,
    #[cfg(feature = "cl1_1")]
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        CommandQueue {
            inner,
            #[cfg(feature = "cl1_1")]
            size: Arc::new(AtomicUsize::new(0))
        }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                self.size.load(std::sync::atomic::Ordering::Relaxed)
            } else {
                0
            }
        }
    }
}

#[cfg(feature = "cl1_1")]
impl CommandQueue {
    pub unsafe fn enqueue_read_buffer (&self, buffer: &RawBuffer, wait: impl Into<WaitList>) -> Result<Eventual> {
        let wait : WaitList = wait.into();
        for evt in wait.wait_all() {

        }

        todo!()
    }
}

#[cfg(not(feature = "cl1_1"))]
impl CommandQueue {
    pub unsafe fn enqueue_read_buffer (&self, wait: impl Into<WaitList>) -> Result<()> {
        todo!()
    }
}