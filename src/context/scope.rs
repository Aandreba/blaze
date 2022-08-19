use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, thread::Thread};
use utils_atomics::FillQueue;

use crate::prelude::{Result, RawCommandQueue, RawEvent};
use super::{Context, Global};

pub struct Scope<'scope, C: ?Sized + Context = Global> {
    ctx: &'scope C,
    thread: Thread,
    pub(super) event_count: Arc<AtomicUsize>,
    pub(super) fallback_events: FillQueue<RawEvent>
}

impl<'scope, C: ?Sized + Context> Scope<'scope, C> {
    pub fn new (ctx: &'scope C) -> Self {
        Self {
            ctx,
            thread: std::thread::current(),
            event_count: Arc::default(),
            fallback_events: FillQueue::new()
        }
    }

    #[inline(always)]
    pub fn enqueue<F: 'scope + FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&'scope self, f: F) -> Result<RawEvent> {
        let evt = self.ctx.next_queue().enqueue_scoped(f)?;
        self.event_count.fetch_add(1, Ordering::AcqRel);

        let my_event_count = self.event_count.clone();
        let my_thread = self.thread.clone();
        let f = move |_, _| if my_event_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            my_thread.unpark();
        };

        // Fallback for callback error. Add event to fill queue
        // This method uses more memory, so it's only used when some error happened setting up the OpenCL callback.
        if evt.on_complete(f).is_err() {
            self.event_count.fetch_sub(1, Ordering::AcqRel);
            self.fallback_events.push(evt.clone());
        }

        Ok(evt)
    }
}