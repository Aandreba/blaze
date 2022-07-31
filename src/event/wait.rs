use std::{sync::Arc, ffi::c_void, task::Poll};
use futures::{task::AtomicWaker, Future, future::FusedFuture};
use opencl_sys::{cl_event, cl_int};
use super::{EventStatus, Event};
use crate::core::*;

/// A future that resolves when it's underlying [`Event`] completes
#[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
#[derive(Clone)]
pub struct EventWait<E: Event> {
    event: Option<E>,
    waker: Arc<AtomicWaker>
}

impl<E: Event + Unpin> EventWait<E> {
    /// Creates a new [`EventWait`] from an [`Event`]
    pub fn new (event: E) -> Result<Self> {
        let waker = Arc::into_raw(Arc::new(AtomicWaker::new()));

        unsafe {
            Arc::increment_strong_count(waker);
            match event.parent_event().on_complete_raw(wake_future, waker as *mut _) {
                Ok(_) => {
                    let waker = Arc::from_raw(waker);
                    Ok(Self { event: Some(event), waker })
                },

                Err(e) => {
                    Arc::decrement_strong_count(waker);
                    Arc::from_raw(waker);
                    Err(e)
                }
            }
        }
    }

    /// Returns the underlying [`Event`]
    /// # Safety
    /// This function will panic if the underlying [`Event`] has already been consumed
    pub fn into_inner (self) -> E {
        self.event.unwrap()
    }
}

impl<E: Event + Unpin> Future for EventWait<E> {
    type Output = Result<E::Output>;

    #[inline]
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let event = self.event.as_ref().unwrap();
        self.waker.register(cx.waker());

        match event.parent_event().status() {
            Ok(EventStatus::Complete) => {
                // SAFETY: We previously checked if the event existed, so it's guaranteed to still be there
                let data = unsafe {
                    core::mem::take(&mut self.event).unwrap_unchecked()
                };

                Poll::Ready(data.consume(None))
            },

            Err(e) => {
                // SAFETY: We previously checked if the event existed, so it's guaranteed to still be there
                let data = unsafe {
                    core::mem::take(&mut self.event).unwrap_unchecked()
                };

                Poll::Ready(data.consume(Some(e)))
            },
            
            _ => Poll::Pending
        }
    }
}

impl<E: Event + Unpin> FusedFuture for EventWait<E> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.event.is_none()
    }
}

#[doc(hidden)]
unsafe extern "C" fn wake_future (_event: cl_event, _event_command_status: cl_int, user_data: *mut c_void) {
    let user_data = Arc::from_raw(user_data as *mut AtomicWaker);
    user_data.wake()
}