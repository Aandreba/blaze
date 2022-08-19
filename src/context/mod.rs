flat_mod!(raw, flags, global, single, queue, scope);

use std::{ops::Deref, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}, sync::atomic::Ordering};
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

    /// Enqueues an event into the provided [`CommandQueue`] by [`next_queue`](Context::next_queue).
    #[inline(always)]
    fn enqueue<F: 'static + FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&self, f: F) -> Result<RawEvent> {
        self.next_queue().enqueue(f)
    }

    #[inline]
    fn scope<'scope, T, F: 'scope + FnOnce(&Scope<'scope, Self>) -> Result<T>> (&'scope self, f: F) -> Result<T> {
        let mut scope = Scope::new(self);
        let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));

        while scope.event_count.load(Ordering::Acquire) != 0 {
            std::thread::park();
        }

        let remaining = scope.fallback_events.chop_mut();
        let remaining = remaining.collect::<Vec<_>>();
        let _ = RawEvent::wait_all(&remaining);

        match result {
            Err(e) => resume_unwind(e),
            Ok(res) => res
        }
    }
}

fn test () {
    let mut alpha = 1;

    std::thread::scope(|s| {
        s.spawn(|| alpha = 2);
    });

    println!("{alpha}");
    
    /*scope(|s| {
        s.spawn(|| alpha = 2);
        s.spawn(|| alpha = 3);
    });*/
    /* 
    let v = Global.scope(|s| {
        s.enqueue(|_| {
            alpha = 2;
            Err(crate::prelude::ErrorType::InvalidValue.into())
        })
    });

    let _ = Global.enqueue(|_| {
        alpha = 3;
        Err(crate::prelude::ErrorType::InvalidValue.into())
    }).unwrap();

    println!("{alpha}");*/
}