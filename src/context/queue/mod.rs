use std::{sync::{Arc}, ops::{Deref, DerefMut}};
use crate::prelude::{RawCommandQueue, RawEvent, Result};

#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    size: Arc<()>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self {
            inner,
            size: Arc::new(())
        }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        Arc::strong_count(&self.size) - 1
    }

    #[inline(always)]
    pub fn enqueue<'a, F: 'a + FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&'a self, f: F) -> Result<RawEvent> {
        self.enqueue_scoped(f)
    }

    #[inline]
    pub(super) fn enqueue_scoped<'a, F: 'a + FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&self, f: F) -> Result<RawEvent> {
        // Generate event
        let event = f(&self)?;

        // Decrement event count when it completes (if callback setup fails, do it now)
        let size = self.size.clone();
        let _ = event.on_complete(move |_, _| drop(size));

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