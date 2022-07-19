use std::{task::{Poll}, time::SystemTime};
use futures::{Future, FutureExt, future::{FusedFuture}};
use crate::prelude::{Result, RawContext, Global};
use super::{FlagEvent, RawEvent, Event};

/// An event that wraps a Rust [`Future`](std::future::Future).
#[cfg_attr(docsrs, doc(cfg(all(feature = "cl1_1", feature = "futures"))))]
#[derive(Debug, Clone)]
pub struct FutureEvent<F: Future> {
    flag: FlagEvent,
    fut: F
}

impl<F: Future + Unpin> FutureEvent<F> {
    #[inline(always)]
    pub fn new (fut: F) -> Result<Self> {
        Self::new_in(&Global, fut)
    }

    #[inline(always)]
    pub fn new_in (ctx: &RawContext, fut: F) -> Result<Self> {
        let flag = FlagEvent::new_in(ctx)?;
        Ok(Self { flag, fut })
    }
}

impl<F: Future + Unpin> Future for FutureEvent<F> {
    type Output = F::Output;

    #[inline(always)]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if let Poll::Ready(x) = self.fut.poll_unpin(cx) {
            self.flag.set_complete(None).unwrap();
            return Poll::Ready(x)
        }

        Poll::Pending
    }
}

impl<F: Future + Unpin> FusedFuture for FutureEvent<F> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.flag.has_completed()
    }
}

impl<F: Future + Unpin> Event for FutureEvent<F> {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.flag.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<crate::prelude::Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) }
        Ok(())
    }

    #[inline(always)]
    fn profiling_nanos (&self) -> Result<super::ProfilingInfo<u64>> {
        self.flag.profiling_nanos()
    }

    #[inline(always)]
    fn profiling_time (&self) -> Result<super::ProfilingInfo<SystemTime>> {
        self.flag.profiling_time()
    }

    #[inline(always)]
    fn duration (&self) -> Result<std::time::Duration> {
        self.flag.duration()
    }
}