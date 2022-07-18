use std::{sync::{Arc, atomic::AtomicUsize}, cell::UnsafeCell};
use once_cell::sync::OnceCell;
use crate::{prelude::{RawCommandQueue, RawEvent, Result}, event::WaitList};

/// A smart command queue. Events pushed to this queue will not be pushed to it's OpenCL counterpart until all
/// their dependants (a.k.a the events in the wait list) have completed.
#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub const fn new (inner: RawCommandQueue) -> Self {
        todo!()
        //Self { inner, buffer }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        self.size.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(feature = "cl1_1")]
    pub fn enqueue<F: FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&self, f: F, wait: impl Into<WaitList>) -> Eventual {
        let wait : WaitList = wait.into();
        let wait = wait.into_vec();

        if wait.len() == 0 {
            let evt = f(&self.inner);
            return Eventual::with_event(evt);
        }

        let remaining = Arc::new(AtomicUsize::new(wait.len()));
        let f = Box::into_raw(Box::new(f)) as usize;
        let result = Arc::new(OnceCell::new());

        for evt in wait {
            let remaining = remaining.clone();
            let inner = self.inner.clone();
            let size = self.size.clone();

            evt.on_complete(move |_, _| {
                if remaining.fetch_sub(1, std::sync::atomic::Ordering::Release) == 1 {
                    let f = unsafe { Box::from_raw(f as *mut F) };
                    let evt = f(&inner);
                }
            });
        }

        todo!()
    }
}

pub struct Eventual {
    evt: Arc<UnsafeCell<T>>,
    #[cfg(feature = "futures")]
    waker: futures::task::AtomicWaker
}

impl Eventual {
    #[inline(always)]
    pub const fn new () -> Self {
        Self { 
            evt: Arc::new(OnceCell::new()),
            #[cfg(feature = "futures")]
            waker: futures::task::AtomicWaker::new()
        }
    }

    #[inline(always)]
    pub const fn with_event (evt: Result<RawEvent>) -> Self {
        Self { 
            evt: Arc::new(OnceCell::with_value(evt)),
            #[cfg(feature = "futures")]
            waker: futures::task::AtomicWaker::new()
        }
    }

    #[inline(always)]
    pub const fn from_cell (evt: Arc<OnceCell<Result<RawEvent>>>) -> Self {
        Self { 
            evt,
            #[cfg(feature = "futures")]
            waker: futures::task::AtomicWaker::new()
        }
    }

    pub fn wait (self) -> Result<RawEvent> {
        self.evt.wait()
    }
}
