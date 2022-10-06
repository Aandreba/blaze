use crate::prelude::*;
use super::{Consumer};
use blaze_proc::docfg;

#[derive(Debug, Clone)]
pub struct Eventual<C, F> {
    parent: Event<C>,
    f: F,
}

impl<N: Consumer, C: Consumer, F: FnOnce(C::Output) -> Result<Event<N>>> Eventual<C, F> {
    #[inline(always)]
    pub const fn new (parent: Event<C>, f: F) -> Self {
        Self { parent, f }
    }

    #[inline(always)]
    pub fn wait (self) -> Result<Event<N>> {
        let v = self.parent.join()?;
        return (self.f)(v);
    }

    #[inline(always)]
    pub fn join (self) -> Result<N::Output> {
        let evt = self.wait()?;
        return evt.join()
    }

    #[docfg(feature = "futures")]
    #[inline(always)]
    pub fn wait_async (self) -> Result<EventualFuture<C, F>> where C: Unpin, F: Unpin {
        let inner = self.parent.join_async()?;
        return Ok(EventualFuture { inner, f: MaybeUninit::new(self.f) })
    }

    #[docfg(feature = "futures")]
    #[inline(always)]
    pub async fn join_async (self) -> Result<N::Output> where C: Unpin, F: Unpin, N: Unpin {
        let v = self.wait_async()?.await?;
        return v.join_async()?.await
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        use futures::future::*;
        use super::EventWait;
        use std::task::*;
        use std::mem::MaybeUninit;

        #[derive(Debug)]
        pub struct EventualFuture<C: Unpin + Consumer, F> {
            inner: EventWait<C>,
            f: MaybeUninit<F>
        }
        
        impl<N: Consumer, C: Consumer + Unpin, F: Unpin + FnOnce(C::Output) -> Result<Event<N>>> Future for EventualFuture<C, F> {
            type Output = Result<Event<N>>;

            #[inline]
            fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                if let Poll::Ready(x) = self.inner.poll_unpin(cx)? {
                    // SAFETY: If `inner` was already consumed, it will already have panicked
                    let f = unsafe { self.f.assume_init_read() };
                    return Poll::Ready(f(x))
                }

                return Poll::Pending;
            }
        }

        impl<C: Unpin + Consumer + Clone, F: Clone> Clone for EventualFuture<C, F> {
            #[inline]
            fn clone (&self) -> Self {
                let f = match FusedFuture::is_terminated(&self.inner) {
                    true => MaybeUninit::uninit(),
                    false => unsafe {
                        MaybeUninit::new(self.f.assume_init_ref().clone())
                    }
                };

                return Self {
                    inner: self.inner.clone(),
                    f
                }
            }
        }

        impl<C: Consumer + Unpin, F> Drop for EventualFuture<C, F> {
            #[inline]
            fn drop(&mut self) {
                if !FusedFuture::is_terminated(&self.inner) {
                    unsafe { self.f.assume_init_drop() }
                }
            }
        }
    }
}