use std::{ffi::c_void, task::{Poll, Waker}, marker::PhantomData};
use futures::{Future, FutureExt, future::FusedFuture};
use opencl_sys::*;
use utils_atomics::{flag::{AsyncFlag, AsyncSubscribe}, FillQueue};
use crate::prelude::Result;
use super::{Event, consumer::Consumer};

/// Future for [`join_async`](super::Event::join_async).
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
#[derive(Debug)]
pub struct EventWait<'a, C: 'a> {
    inner: Option<Event<C>>,
    sub: AsyncSubscribe,
    phtm: PhantomData<&'a ()>
}

impl<'a, C: Unpin + Consumer<'a>> EventWait<'a, C> {
    #[inline(always)]
    pub fn new (inner: Event<C>) -> Result<Self> {
        let flag = AsyncFlag::new();
        let sub = flag.subscribe();
        
        unsafe {
            inner.on_complete_raw(wake_future, flag.into_raw() as *mut _)?;
        }

        return Ok(Self { inner: Some(inner), sub, phtm: PhantomData })
    }
}

impl<'a, C: Clone + Consumer<'a>> Clone for EventWait<'a, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self { 
            inner: self.inner.clone(),
            sub: self.sub.clone(),
            phtm: self.phtm.clone()
        }
    }
}

impl<'a, C: Unpin + Consumer<'a>> Future for EventWait<'a, C> {
    type Output = Result<C::Output>;

    #[inline(always)]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let this = &mut *self;
        if this.sub.poll_unpin(cx).is_ready() {
            let event = this.inner.take().unwrap();
            return Poll::Ready(event.status().and_then(|_| event.consume()))
        }

        return Poll::Pending;
    }
}

impl<'a, C: Unpin + Consumer<'a>> FusedFuture for EventWait<'a, C> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.inner.is_none()
    }
}

#[doc(hidden)]
unsafe extern "C" fn wake_future (_event: cl_event, _event_command_status: cl_int, user_data: *mut c_void) {
    let _ = AsyncFlag::from_raw(user_data as *const FillQueue<Waker>);
}