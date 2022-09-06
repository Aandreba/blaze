use std::{ffi::c_void, task::{Poll, Waker}};
use futures::{Future, FutureExt};
use opencl_sys::*;
use utils_atomics::{flag::{AsyncFlag, AsyncSubscribe}, FillQueue};
use crate::prelude::Result;
use super::{Event, EventStatus};

#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
pub struct EventWait<'a, T> {
    inner: Option<Event<'a, T>>,
    sub: AsyncSubscribe
}

impl<'a, T> EventWait<'a, T> {
    #[inline(always)]
    pub fn new (inner: Event<'a, T>) -> Result<Self> {
        Self::on_status(inner, EventStatus::Complete)
    }

    #[inline]
    pub fn on_status (inner: Event<'a, T>, status: EventStatus) -> Result<Self> {
        let flag = AsyncFlag::new();
        let sub = flag.subscribe();
        
        unsafe {
            inner.on_status_raw(status, wake_future, flag.into_raw() as *mut _)?;
        }

        return Ok(Self { inner: Some(inner), sub })
    }
}

impl<'a, T> Future for EventWait<'a, T> {
    type Output = Result<T>;

    #[inline(always)]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.sub.poll_unpin(cx).is_ready() {
            let event = self.inner.take().unwrap();
            return Poll::Ready(event.status().and_then(|_| event.consume()))
        }

        return Poll::Pending;
    }
}

#[doc(hidden)]
unsafe extern "C" fn wake_future (_event: cl_event, _event_command_status: cl_int, user_data: *mut c_void) {
    let _ = AsyncFlag::from_raw(user_data as *const FillQueue<Waker>);
}