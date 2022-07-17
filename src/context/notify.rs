use std::sync::{Arc, atomic::AtomicUsize};
use crate::prelude::{RawEvent, Result};

/// Notifier
#[derive(Debug)]
#[repr(transparent)]
pub struct Notify (Arc<AtomicUsize>);

impl Notify {
    #[inline(always)]
    pub const fn new (count: Arc<AtomicUsize>) -> Self {
        Self(count)
    }

    #[inline(always)]
    pub fn notify (self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Release);
    }

    #[inline(always)]
    pub fn bind (self, evt: &RawEvent) {
        let count = self.0.clone();
        if evt.on_complete(move |_, _| { count.fetch_sub(1, std::sync::atomic::Ordering::Release); }).is_err() {
            self.notify()
        }
    }

    #[inline(always)]
    pub fn bind_result (self, evt: Result<RawEvent>) -> Result<RawEvent> {
        match evt {
            Ok(evt) => {
                self.bind(&evt);
                Ok(evt)
            },

            Err(e) => {
                self.notify();
                Err(e)
            }
        }
    }
}