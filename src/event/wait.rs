use std::{ffi::c_void, task::{Poll, Waker}};
use futures::{Future, FutureExt, future::FusedFuture};
use opencl_sys::*;
use utils_atomics::{flag::{AsyncFlag, AsyncSubscribe}, FillQueue};
use crate::prelude::Result;
use super::{Event, consumer::Consumer};

/// Future for [`join_async`](super::Event::join_async).
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
#[derive(Debug, Clone)]
pub struct EventWait<C> {
    inner: Option<Event<C>>,
    sub: AsyncSubscribe
}

impl<C: Unpin + Consumer> EventWait<C> {
    #[inline(always)]
    pub fn new (inner: Event<C>) -> Result<Self> {
        let flag = AsyncFlag::new();
        let sub = flag.subscribe();
        
        unsafe {
            inner.on_complete_raw(wake_future, flag.into_raw() as *mut _)?;
        }

        return Ok(Self { inner: Some(inner), sub })
    }
}

impl<C: Unpin + Consumer> Future for EventWait<C> {
    type Output = Result<C::Output>;

    #[inline(always)]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let this = &mut *self;
        if this.sub.poll_unpin(cx).is_ready() {
            let event = this.inner.take().unwrap();
            return Poll::Ready(event.status().and_then(|_| unsafe { event.consume() }))
        }

        return Poll::Pending;
    }
}

impl<C: Unpin + Consumer> FusedFuture for EventWait<C> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.inner.is_none()
    }
}

#[doc(hidden)]
unsafe extern "C" fn wake_future (_event: cl_event, _event_command_status: cl_int, user_data: *mut c_void) {
    let _ = AsyncFlag::from_raw(user_data as *const FillQueue<Waker>);
}