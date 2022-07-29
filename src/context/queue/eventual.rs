use std::{sync::{Arc, atomic::{AtomicPtr, AtomicI32}}, ffi::c_void};
use blaze_proc::docfg;
use crossbeam::{queue::SegQueue, atomic::AtomicConsume};
use crate::prelude::{RawEvent, Result, Error, Event};

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        enum EventualWaker {
            Sync (std::thread::Thread),
            Async (core::task::Waker)
        }

        impl EventualWaker {
            #[inline(always)]
            pub fn wake (self) {
                match self {
                    Self::Sync (x) => x.unpark(),
                    Self::Async (x) => x.wake(),
                }
            }
        }

        impl From<std::thread::Thread> for EventualWaker {
            #[inline(always)]
            fn from (x: std::thread::Thread) -> Self {
                Self::Sync(x)
            }            
        }

        impl From<core::task::Waker> for EventualWaker {
            #[inline(always)]
            fn from (x: core::task::Waker) -> Self {
                Self::Async(x)
            }            
        }
    } else {
        #[repr(transparent)]
        struct EventualWaker (std::thread::Thread);

        impl EventualWaker {
            #[inline(always)]
            pub fn wake (&self) {
                self.0.unpark()
            }
        }

        impl From<std::thread::Thread> for EventualWaker {
            #[inline(always)]
            fn from (x: std::thread::Thread) -> Self {
                Self(x)
            }            
        }
    }
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Eventual (Arc<EventualInner>);

impl Eventual {
    #[inline(always)]
    pub fn new () -> Self {
        Self(Arc::new(EventualInner::new()))
    }

    #[inline(always)]
    pub fn from_event (evt: RawEvent) -> Self {
        Self(Arc::new(EventualInner::from_event(evt)))
    }

    #[inline(always)]
    pub fn from_error (err: Error) -> Self {
        Self(Arc::new(EventualInner::from_error(err.ty)))
    }

    #[inline]
    pub fn try_get (&self) -> Option<RawEvent> {
        let evt = RawEvent::from_id(self.0.inner.load_consume())?;
        unsafe { evt.retain().unwrap() };
        Some(evt)
    }

    #[inline(always)]
    pub unsafe fn get_unchecked (&self) -> Result<RawEvent> {
        match self.0.status.load_consume() {
            0 => {
                let evt = RawEvent::from_id_unchecked(self.0.inner.load_consume());
                evt.retain().unwrap();
                Ok(evt)
            },

            other => Err(Error::from(other))
        }
    }

    #[inline(always)]
    pub fn wait (&self) -> Result<RawEvent> {
        self.0.register(std::thread::current());
        std::thread::park();

        unsafe {
            self.get_unchecked()
        }
    }

    #[docfg(feature = "futures")]
    pub fn wait_async (&self) -> RawEvent {
        todo!()
    }

    #[inline]
    pub(super) unsafe fn set_unchecked (&self, evt: Result<RawEvent>) {
        match evt {
            Ok(evt) => {
                self.0.inner.store(evt.id(), std::sync::atomic::Ordering::Release);
                self.0.status.store(0, std::sync::atomic::Ordering::Release);
            },

            Err(err) => {
                self.0.status.store(err.ty as i32, std::sync::atomic::Ordering::Release)
            }
        }

        while let Some(waker) = self.0.wakers.pop() {
            waker.wake();
        }
    }
}

struct EventualInner {
    inner: AtomicPtr<c_void>,
    status: AtomicI32, // 2 = uninit, 1 = working, 0 = completed, -1.. = error
    wakers: SegQueue<EventualWaker>
}

impl EventualInner {
    #[inline(always)]
    fn new () -> Self {
        Self { 
            inner: AtomicPtr::default(),
            status: AtomicI32::new(2),
            wakers: SegQueue::new()
        }
    }

    #[inline(always)]
    fn from_event (evt: RawEvent) -> Self {
        Self { 
            inner: AtomicPtr::new(evt.id()),
            status: AtomicI32::default(),
            wakers: SegQueue::new()
        }
    }

    #[inline(always)]
    fn from_error (err: crate::core::ErrorType) -> Self {
        Self {
            inner: AtomicPtr::default(),
            status: AtomicI32::new(err as i32),
            wakers: SegQueue::new()
        }
    }

    #[inline(always)]
    fn register (&self, waker: impl Into<EventualWaker>) {
        if self.status.load_consume() < 1 {
            waker.into().wake();
            return;
        }

        self.wakers.push(waker.into())
    }
}

#[docfg(feature = "futures")]
impl futures::Future for Eventual {
    type Output = RawEvent;

    #[inline(always)]
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.0.register(cx.waker().clone());
        if let Some(evt) = self.try_get() {
            return std::task::Poll::Ready(evt);
        }

        std::task::Poll::Pending
    }
}

unsafe impl Send for EventualInner {}
unsafe impl Sync for EventualInner {}