use std::{sync::{Arc, atomic::{AtomicU8, Ordering}}, hint::unreachable_unchecked};
use crate::prelude::Result;
use super::{FlagEvent, _consumer::Consumer};

pub(super) const UNINIT : u8 = 2;
const TRUE : u8 = 1;
pub(super) const FALSE : u8 = 0;

/// Handle to abort an [`AbortableEvent`](super::consumer::AbortableEvent).
#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[derive(Debug, Clone)]
pub struct AbortHandle {
    pub(super) inner: FlagEvent,
    pub(super) aborted: Arc<AtomicU8>
}

impl AbortHandle {
    /// Attempts to abort it's assigned event, returning `true` when successfully aborted and `false` when the event has already completed or been aborted.
    #[inline(always)]
    pub fn try_abort (&self) -> Result<bool> {
        if self.inner.try_mark(None)? {
            self.aborted.store(TRUE, Ordering::Release);
            return Ok(true);
        }

        return Ok(false)
    }
}

/// Consumer for [`abortable`](super::Event::abortable) event.
#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
pub struct Abort<C> {
    pub(super) aborted: Arc<AtomicU8>,
    pub(super) consumer: C
}

impl<C: Consumer> Consumer for Abort<C> {
    type Output = Option<C::Output>;

    #[inline]
    fn consume (self) -> Result<Self::Output> {
        loop {
            match self.aborted.load(Ordering::Acquire) {
                TRUE => return Ok(None),
                FALSE => return self.consumer.consume().map(Some),
                UNINIT => core::hint::spin_loop(),
                _ => unsafe { unreachable_unchecked() }
            }
        }
    }
}