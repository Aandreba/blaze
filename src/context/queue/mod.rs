use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, ops::{Deref, DerefMut}};
use crate::prelude::{RawCommandQueue, RawEvent, Result};

#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self {
            inner,
            size: Arc::new(AtomicUsize::default())
        }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    #[inline]
    pub fn enqueue<F: FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&self, f: F) -> Result<RawEvent> {
        // Generate event
        let event = f(&self)?;
        self.size.fetch_add(1, Ordering::AcqRel);

        // Keep track of queue size
        let size = self.size.clone();
        if event.on_complete(move |_, _| {size.fetch_sub(1, Ordering::AcqRel);}).is_err() {
            self.size.fetch_sub(1, Ordering::AcqRel);
        }

        Ok(event)
    }
}

impl Deref for CommandQueue {
    type Target = RawCommandQueue;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CommandQueue {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}