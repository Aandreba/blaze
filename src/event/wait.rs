use std::{ffi::c_void, task::{Poll, Waker}};
use futures::{Future, FutureExt};
use opencl_sys::*;
use utils_atomics::{flag::{AsyncFlag, AsyncSubscribe}, FillQueue};
use crate::prelude::Result;
use super::{Event, consumer::Consumer};

#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
pub struct EventWait<T, C> {
    inner: Option<Event<T, C>>,
    sub: AsyncSubscribe
}

impl<'a, T, C: Unpin + Consumer<'a, T>> EventWait<T, C> {
    #[inline(always)]
    pub fn new (inner: Event<T, C>) -> Result<Self> {
        let flag = AsyncFlag::new();
        let sub = flag.subscribe();
        
        unsafe {
            inner.on_complete_raw(wake_future, flag.into_raw() as *mut _)?;
        }

        return Ok(Self { inner: Some(inner), sub })
    }
}

impl<'a, T, C: Unpin + Consumer<'a, T>> Future for EventWait<T, C> {
    type Output = Result<T>;

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

#[doc(hidden)]
unsafe extern "C" fn wake_future (_event: cl_event, _event_command_status: cl_int, user_data: *mut c_void) {
    let _ = AsyncFlag::from_raw(user_data as *const FillQueue<Waker>);
}