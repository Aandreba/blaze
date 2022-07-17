use std::{sync::{atomic::{AtomicUsize}, Arc}, pin::Pin, mem::{MaybeUninit, ManuallyDrop}, backtrace::Backtrace, ptr::addr_of, ops::Deref};
use bitvec::{prelude::BitBox, bits, bitbox, ptr::BitRef};
use once_cell::sync::OnceCell;

use crate::{prelude::{RawContext, Result, Global, Error}};
use super::{RawEvent, Event, FlagEvent};

/// Event for [`EventExt::join`](super::EventExt::join)
pub struct EventJoin<E: Event> where E::Output: Unpin  {
    data: Pin<Arc<JoinEventInner<E>>>
}

struct JoinEventInner<E: Event> where E::Output: Unpin  {
    flag: FlagEvent,
    results: JoinList<E>,
    error: OnceCell<JoinError>
}

struct JoinList<E: Event> where E::Output: Unpin {
    inner: Pin<Box<[MaybeUninit<E::Output>]>>,
    idx: AtomicUsize
}

// Error
struct JoinError {
    desc: Option<String>,
    #[cfg(debug_assertions)]
    backtrace: Arc<Backtrace>
}

impl<E: Event> EventJoin<E> where E: 'static + Send, E::Output: Send + Sync + Unpin {
    #[inline(always)]
    pub fn new<I: IntoIterator<Item = E>> (events: I) -> Result<Self> where I::IntoIter: ExactSizeIterator {
        Self::new_in(&Global, events)
    }

    pub fn new_in<I: IntoIterator<Item = E>> (ctx: &RawContext, events: I) -> Result<Self> where I::IntoIter: ExactSizeIterator {
        let events = events.into_iter();
        
        let results = JoinList {
            inner: Pin::new(Box::new_uninit_slice(events.len())),
            idx: AtomicUsize::new(0),
        };

        let data = Arc::pin(JoinEventInner {
            flag: FlagEvent::new_in(ctx)?,
            error: OnceCell::new(),
            results
        });

        for event in events {
            let data = data.clone();
            let raw = event.as_raw().clone();

            raw.on_complete(move |_, status| {
                match event.consume(status.err()) {
                    Ok(v) => if data.results.push(v) {
                        data.flag.set_complete(None).unwrap();
                    },

                    Err(err) => {
                        let set = data.error.set(JoinError {
                            desc: err.desc,
                            #[cfg(debug_assertions)]
                            backtrace: err.backtrace
                        });

                        if set.is_ok() {
                            data.flag.set_complete(Some(err.ty)).unwrap();
                        }
                    }
                }
            })?;
        }

        Ok(Self { data })
    }
}

impl<E: Event> Event for EventJoin<E> where E::Output: Unpin {
    type Output = Vec<E::Output>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.data.flag.as_raw()
    }

    fn consume (self, err: Option<crate::prelude::Error>) -> crate::prelude::Result<Self::Output> {
        let mut unpined = Pin::into_inner(self.data);
        let data;

        loop {
            match Arc::try_unwrap(unpined) {
                Ok(x) => {
                    data = x;
                    break
                },

                Err(x) => {
                    unpined = x;
                    core::hint::spin_loop()
                }
            }
        }

        if let Some(err) = err {
            let data = data.error.into_inner().unwrap();
            return Err(Error::from_parts(err.ty, data.desc, #[cfg(debug_assertions)] data.backtrace));
        }

        unsafe { Ok(data.results.assume_init()) }
    }

    #[inline(always)]
    fn profiling_nanos (&self) -> Result<super::ProfilingInfo<u64>> {
        self.data.flag.profiling_nanos()
    }

    #[inline(always)]
    fn profiling_time (&self) -> Result<super::ProfilingInfo<std::time::SystemTime>> {
        self.data.flag.profiling_time()
    }

    #[inline(always)]
    fn duration (&self) -> Result<std::time::Duration> {
        self.data.flag.duration()
    }
}

impl<E: Event> JoinList<E> where E::Output: Unpin {
    fn push (&self, v: E::Output) -> bool {
        let idx = self.idx.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        debug_assert!(idx < self.inner.len());

        // SAFETY: Atomically increased index ensures we are the only thread with access to this index.
        unsafe {
            let ptr = self.inner.as_ptr().add(idx) as *mut MaybeUninit<E::Output>;
            (&mut *ptr).write(v) 
        };

        idx == self.inner.len() - 1
    }

    unsafe fn assume_init (self) -> Vec<E::Output> {
        let this = ManuallyDrop::new(self);
        let inner = core::ptr::read(addr_of!(this.inner));

        let results = Pin::into_inner(inner);
        results.assume_init().into_vec()
    }
}

impl<E: Event> Drop for JoinList<E> where E::Output: Unpin {
    #[inline]
    fn drop(&mut self) {
        let ptr = self.inner.as_mut_ptr();
        for i in 0..*self.idx.get_mut() {
            unsafe { (&mut *ptr.add(i)).assume_init_drop() }
        }
    }
}

/// Event for [`EventExt::join_ordered`](super::EventExt::join_ordered)
pub struct EventJoinOrdered<E: Event> where E::Output: Unpin  {
    data: Pin<Arc<JoinOrderedInner<E>>>
}

struct JoinOrderedInner<E: Event> where E::Output: Unpin {
    flag: FlagEvent,
    results: JoinOrderedList<E>,
    error: OnceCell<JoinError>
}

struct JoinOrderedList<E: Event> where E::Output: Unpin {
    inner: Pin<Box<[MaybeUninit<E::Output>]>>,
    has_init: BitBox
}

impl<E: Event> EventJoinOrdered<E> where E: 'static + Send, E::Output: Send + Sync + Unpin {
    #[inline(always)]
    pub fn new<I: IntoIterator<Item = E>> (events: I) -> Result<Self> where I::IntoIter: ExactSizeIterator {
        Self::new_in(&Global, events)
    }

    pub fn new_in<I: IntoIterator<Item = E>> (ctx: &RawContext, events: I) -> Result<Self> where I::IntoIter: ExactSizeIterator {
        let events = events.into_iter();

        let results = JoinOrderedList {
            inner: Pin::new(Box::new_uninit_slice(events.len())),
            has_init: bitbox![0; events.len()]
        };

        let data = Arc::pin(JoinOrderedInner {
            flag: FlagEvent::new_in(ctx)?,
            error: OnceCell::new(),
            results
        });

        for (idx, event) in events.enumerate() {
            let data = data.clone();
            let raw = event.as_raw().clone();

            raw.on_complete(move |_, status| {
                match event.consume(status.err()) {
                    Ok(v) => if data.results.set(idx, v) {
                        data.flag.set_complete(None).unwrap();
                    },

                    Err(err) => {
                        let set = data.error.set(JoinError {
                            desc: err.desc,
                            #[cfg(debug_assertions)]
                            backtrace: err.backtrace
                        });

                        if set.is_ok() {
                            data.flag.set_complete(Some(err.ty)).unwrap();
                        }
                    }
                }
            })?;
        }

        Ok(Self { data })
    }
}

impl<E: Event> JoinOrderedList<E> where E::Output: Unpin {
    #[inline]
    fn set (&self, idx: usize, v: E::Output) -> bool {
        let box_ptr = self.inner.as_ptr() as *mut MaybeUninit<E::Output>;
        let bit_ptr = addr_of!(self.has_init) as *mut BitBox;

        // SAFETY: This private code is guaranteed to not write to the some pointer on multiple ocasions in the implementation of `EventJoinOrdered`
        unsafe {
            (&mut *box_ptr).write(v);
            (&mut *bit_ptr).as_mut_bitptr().add(idx).write(true);
        }

        self.has_init.iter().all(|x| *x)
    }
}

impl<E: Event> Drop for JoinOrderedList<E> where E::Output: Unpin {
    #[inline]
    fn drop(&mut self) {
        let inner = Pin::into_inner(self.inner);
        for (init, inner) in self.has_init.into_iter().zip(inner.iter_mut()) {
            if init {
                unsafe { inner.assume_init_drop(); }
            }
        }
    }
}