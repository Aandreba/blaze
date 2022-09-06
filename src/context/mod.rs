flat_mod!(raw, flags, global, single, queue);

use std::ops::Deref;
use crate::prelude::Result;

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