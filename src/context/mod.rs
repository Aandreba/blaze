flat_mod!(raw, flags, global, single, queue, scope);

use std::{ops::Deref, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}};
use utils_atomics::FillQueue;
use crate::{core::*, prelude::RawEvent};

/// An object that can be used as a Blaze context, with a similar syntax to Rust allocators.\
/// Blaze contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows Blaze contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context: Deref<Target = RawContext> {
    fn queues (&self) -> &[CommandQueue];
    fn next_queue (&self) -> &CommandQueue;

    #[inline(always)]
    fn as_raw (&self) -> &RawContext {
        self
    }

    #[inline(always)]
    fn flush_all (&self) -> Result<()> {
        for queue in self.queues() {
            queue.flush()?
        }

        Ok(())
    }

    #[inline(always)]
    fn finish_all (&self) -> Result<()> {
        for queue in self.queues() {
            queue.finish()?
        }

        Ok(())
    }

    #[inline]
    fn scope<'scope, T, F: FnOnce(&'scope Scope<'scope, Self>) -> T > (&'scope self, f: F) -> Result<T> {
        let scope = Scope {
            ctx: self,
            events: FillQueue::new()
        };

        let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));
        let events = scope.events.chop_mut().collect::<Vec<_>>();
        
        match (result, RawEvent::wait_all(&events)) {
            (Err(e), _) => resume_unwind(e),
            (_, Err(e)) => return Err(e),
            (Ok(x), _) => return Ok(x)
        }
    }

    #[inline(always)]
    fn enqueue<F: FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&self, f: F) -> Result<RawEvent> {
       self.next_queue().enqueue(f)
    }
}