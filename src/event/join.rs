use std::{sync::{atomic::{AtomicUsize}, Arc}, pin::Pin, mem::{MaybeUninit, ManuallyDrop}, backtrace::Backtrace, ptr::addr_of};
use once_cell::sync::OnceCell;

use crate::{prelude::{RawContext, Result, Global, Error}};
use super::{RawEvent, Event, FlagEvent};

pub struct EventJoinUnordered<E: Event> where E::Output: Unpin  {
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

struct JoinError {
    desc: Option<String>,
    #[cfg(debug_assertions)]
    backtrace: Arc<Backtrace>
}

impl<E: Event> EventJoinUnordered<E> where E: 'static + Send, E::Output: 'static + Send + Sync + Unpin {
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
                        data.flag.set_complete(None);
                    },

                    Err(err) => {
                        let set = data.error.set(JoinError {
                            desc: err.desc,
                            #[cfg(debug_assertions)]
                            backtrace: err.backtrace
                        });

                        if set.is_ok() {
                            data.flag.set_complete(Some(err.ty));
                        }
                    }
                }
            })?;
        }

        Ok(Self { data })
    }
}

impl<E: Event> Event for EventJoinUnordered<E> where E::Output: Unpin {
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