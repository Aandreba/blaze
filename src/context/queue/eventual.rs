use std::time::SystemTime;
use crate::{event::{ProfilingInfo, WaitList}, prelude::{RawEvent, Event, Error, Result}};
use super::{CommandQueue, EventSupplier};

pub type DynEventual = Eventual<Box<dyn EventSupplier>>;

#[cfg(feature = "cl1_1")]
pub struct Eventual<F> {
    join: crate::event::EventJoin<RawEvent>,
    parent: CommandQueue,
    wait: WaitList,
    f: F
}

#[cfg(not(feature = "cl1_1"))]
pub struct Eventual<F> {
    inner: Result<RawEvent>,
    phtm: std::marker::PhantomData<F>
}

impl<F: EventSupplier> Eventual<F> {
    #[inline(always)]
    #[cfg(feature = "cl1_1")]
    pub(super) fn new (join: crate::event::EventJoin<RawEvent>, parent: CommandQueue, f: F, wait: WaitList) -> Self {
        Self { join, parent, f, wait }
    }

    #[inline(always)]
    #[cfg(not(feature = "cl1_1"))]
    pub(super) fn new (queue: &CommandQueue, f: F, wait: WaitList) -> Self {
        let inner = f(queue, wait.into());
        Self { inner, phtm: std::marker::PhantomData }
    }

    #[inline(always)]
    pub fn into_dyn (self) -> DynEventual where F: 'static {
        #[cfg(feature = "cl1_1")]
        return Eventual { join: self.join, parent: self.parent, wait: self.wait, f: Box::new(self.f) };
        #[cfg(not(feature = "cl1_1"))]
        Eventual { inner: self.inner, phtm: std::marker::PhantomData }
    }
}

impl<F: EventSupplier> Event for Eventual<F> {
    type Output = RawEvent;

    #[cfg(feature = "cl1_1")]
    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.join.as_raw()
    }

    #[cfg(not(feature = "cl1_1"))]
    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        todo!()
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }

        match self.f.supply(&self.parent.inner, self.wait) {
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

pub enum BlazeEvent<F> {
    Eventual (Eventual<F>),
    Event (Result<RawEvent>)
}

impl<F: EventSupplier> Event for BlazeEvent<F> {
    type Output;

    fn as_raw (&self) -> &RawEvent {
        todo!()
    }

    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        todo!()
    }
}