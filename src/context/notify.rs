use std::sync::{Arc, atomic::AtomicUsize};
use crate::prelude::{RawEvent, Result, Event};

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
    pub fn bind_result<T: Event> (self, evt: Result<T>) -> Result<T> {
        match evt {
            Ok(evt) => {
                self.bind(evt.as_raw());
                Ok(evt)
            },

            Err(e) => {
                self.notify();
                Err(e)
            }
        }
    }
}

#[inline(always)]
pub fn bind_result<T: Event> (notify: Option<Notify>, evt: Result<T>) -> Result<T> {
    if let Some(notify) = notify {
        return notify.bind_result(evt)
    }

    evt
}