use std::{sync::{Arc, atomic::{AtomicUsize}}, time::SystemTime};
use crate::{prelude::{RawCommandQueue, RawEvent, Result, Error, Event}, event::{WaitList, EventJoin, ProfilingInfo}};

pub type DynEventual = Eventual<Box<dyn FnOnce(&RawCommandQueue) -> Result<RawEvent>>>;

/// A smart command queue. Events pushed to this queue will not be pushed to it's OpenCL counterpart until all
/// their dependants (a.k.a the events in the wait list) have completed.
#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    #[cfg(feature = "cl1_1")]
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        todo!()
        //Self { inner, buffer }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        #[cfg(feature = "cl1_1")]
        return self.size.load(std::sync::atomic::Ordering::Relaxed);
        #[cfg(not(feature = "cl1_1"))]
        0
    }

    #[cfg(feature = "cl1_1")]
    #[inline]
    pub fn enqueue<F: FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> (&self, f: F, wait: impl Into<WaitList>) -> Result<Eventual<F>> {
        self.size.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let wait : WaitList = wait.into();
        let ctx = self.inner.context()?;

        let join = <RawEvent as crate::prelude::EventExt>::join_in(&ctx, wait.iter().cloned())?;
        Ok(Eventual::new(join, self.clone(), f))
    }

    #[cfg(not(feature = "cl1_1"))]
    #[inline(always)]
    pub fn enqueue<F: FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> (&self, f: F, wait: impl Into<WaitList>) -> Result<Eventual<F>> {
        Ok(Eventual::new(&self.inner, f, ))
    }
}

#[cfg(feature = "cl1_1")]
pub struct Eventual<F> {
    join: EventJoin<RawEvent>,
    parent: CommandQueue,
    wait: WaitList,
    f: F
}

#[cfg(not(feature = "cl1_1"))]
pub struct Eventual<F> {
    inner: Result<RawEvent>,
    phtm: std::marker::PhantomData<F>
}

impl<F: FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> Eventual<F> {
    #[inline(always)]
    #[cfg(feature = "cl1_1")]
    fn new (join: EventJoin<RawEvent>, parent: CommandQueue, f: F) -> Self {
        Self { join, parent, f }
    }

    #[inline(always)]
    #[cfg(not(feature = "cl1_1"))]
    fn new (queue: &CommandQueue, f: F, wait: impl Into<WaitList>) -> Self {
        let inner = f(queue, wait.into());
        Self { inner, phtm: std::marker::PhantomData }
    }
}

impl<F: FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> Event for Eventual<F> {
    type Output = RawEvent;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.join.as_raw()
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }

        match (self.f)(&self.parent.inner) {
            Ok(evt) => {
                if evt.on_complete(move |_, _| { self.parent.size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed); }).is_err() {
                    self.parent.size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                }

                Ok(evt)
            },

            Err(e) => {
                self.parent.size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                Err(e)
            }
        }
    }

    #[inline(always)]
    fn profiling_nanos (&self) -> Result<ProfilingInfo<u64>> {
        self.join.profiling_nanos()
    }

    #[inline(always)]
    fn profiling_time (&self) -> Result<ProfilingInfo<SystemTime>> {
        self.join.profiling_time()
    }

    #[inline(always)]
    fn duration (&self) -> Result<std::time::Duration> {
        self.join.duration()
    }
}