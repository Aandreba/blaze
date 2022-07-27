flat_mod!(raw,flags, global, single);

use std::ops::Deref;
use crate::core::*;

/// An object that can be used as a RSCL context, with a similar syntax to Rust allocators.\
/// RSCL contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows RSCL contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context: Deref<Target = RawContext> {
    fn queues (&self) -> &[RawCommandQueue];
    fn next_queue (&self) -> &RawCommandQueue;

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
}