use std::{sync::Arc, rc::Rc};
use crate::prelude::Result;

flat_mod!(scope, raw, flags, global, single, queue);

/// An object that can be used as a Blaze context, with a similar syntax to Rust allocators.\
/// Blaze contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows Blaze contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context {
    fn as_raw (&self) -> &RawContext;
    fn queues (&self) -> &[CommandQueue];
    fn next_queue (&self) -> &CommandQueue;

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
}

impl<T: Context> Context for Rc<T> {
    #[inline(always)]
    fn as_raw (&self) -> &RawContext {
        T::as_raw(self)
    }

    #[inline(always)]
    fn queues (&self) -> &[CommandQueue] {
        T::queues(self)
    }

    #[inline(always)]
    fn next_queue (&self) -> &CommandQueue {
        T::next_queue(self)
    }
}

impl<T: Context> Context for Arc<T> {
    #[inline(always)]
    fn as_raw (&self) -> &RawContext {
        T::as_raw(self)
    }

    #[inline(always)]
    fn queues (&self) -> &[CommandQueue] {
        T::queues(self)
    }

    #[inline(always)]
    fn next_queue (&self) -> &CommandQueue {
        T::next_queue(self)
    }
}