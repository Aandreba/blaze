use std::{sync::{atomic::{AtomicI32, Ordering}}, mem::MaybeUninit, cell::UnsafeCell, thread::Thread, collections::VecDeque};
use crate::prelude::*;

const UNINIT : i32 = 2;
const WORKING : i32 = 1;
const OK : i32 = 0;

pub struct Eventual {
    state: AtomicI32,
    inner: UnsafeCell<MaybeUninit<RawEvent>>,
    queue: MaybeUninit<VecDeque<Waker>>
}

impl Eventual {
    #[inline(always)]
    pub fn new () -> Self {
        Self {
            state: AtomicI32::new(UNINIT),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            queue: MaybeUninit::new(VecDeque::default())
        }
    }

    #[inline]
    pub fn get (&self) -> Result<&RawEvent> {
        self.state.compare_exchange(UNINIT, WORKING, success, failure);
        todo!()
    }

    #[inline]
    pub fn try_get (&self) -> Option<Result<&RawEvent>> {
        match self.state.load(Ordering::Acquire) {
            UNINIT | WORKING => None,
            OK => unsafe {
                let rf = (&*self.inner.get()).assume_init_ref();
                Some(Ok(rf))
            },

            err => Some(Err(Error::from(err)))
        }
    }

    #[inline(always)]
    pub(super) fn try_init_event (&self, v: RawEvent) {
        match self.state.compare_exchange(UNINIT, WORKING, Ordering::AcqRel, Ordering::Relaxed) {
            Ok(_) => unsafe {
                (&mut *self.inner.get()).write(v);
                self.wake_all();
                self.state.store(OK, Ordering::Release);
            },

            _ => {}
        }
    }

    #[inline(always)]
    pub(super) fn try_init_error (&self, err: ErrorType) {
        if self.state.compare_exchange(UNINIT, WORKING, Ordering::AcqRel, Ordering::Relaxed).is_err() {
            return;
        }

        self.wake_all();
        self.state.store(err as i32, Ordering::Release);
    }

    #[inline]
    fn wake_all (&self) {
        self.queue
    }
}

impl Drop for Eventual {
    #[inline(always)]
    fn drop(&mut self) {
        match *self.state.get_mut() {
            UNINIT => {},
            WORKING => panic!("Shouldn't be dropping an Eventual that's initializing"),

            // OK or error
            _ => unsafe {
                self.inner.get_mut().assume_init_drop();
                self.queue.assume_init_drop();
            },
        }
    }
}

unsafe impl Send for Eventual {}
unsafe impl Sync for Eventual {}

enum Waker {
    Sync (Thread),
    #[cfg(feature = "futures")]
    Async (std::task::Waker)
}

impl Waker {
    #[inline(always)]
    pub fn wake (self) {
        match self {
            Self::Sync(x) => x.unpark(),
            #[cfg(feature = "futures")]
            Self::Async(x) => x.wake()
        }
    }
}