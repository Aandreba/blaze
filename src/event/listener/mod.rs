#![allow(unused)]

use std::{collections::{HashMap}, ffi::c_void, sync::{Mutex}, ops::{Deref, DerefMut}};
use crate::prelude::*;
use super::EventStatus;
use once_cell::sync::Lazy;
use opencl_sys::*;

flat_mod!(ty, thread, queue);

type ListenerQueue = HashMap<RawEvent, Vec<Listener>>;
cfg_if::cfg_if! {
    if #[cfg(target_has_atomic = "8")] {
        type Flag = std::sync::atomic::AtomicBool;
        const TRUE : bool = true;
        const FALSE : bool = false;
    } else {
        type Flag = std::sync::atomic::AtomicUsize;
        const TRUE : usize = 1;
        const FALSE : usize = 0;
    }
}

lazy_static! {
    static ref SUBMITTING : Mutex<ListenerQueue> = Mutex::new(ListenerQueue::new());
    static ref RUNNING : Mutex<ListenerQueue> = Mutex::new(ListenerQueue::new());
    static ref COMPLETED : Mutex<ListenerQueue> = Mutex::new(ListenerQueue::new());
}

static EVENTS : [Lazy<Mutex<ListenerQueue>>; 3] = [COMPLETED, RUNNING, SUBMITTING];

fn add_listener (evt: RawEvent, status: EventStatus, listener: Listener) -> Result<()> {
    if status.is_queued() {
        return Err(Error::new(ErrorType::InvalidValue, "Cannot listen to 'Queued' status"))
    }

    init_thread();

    let events = Lazy::force(&EVENTS[status as usize]);
    let events = events.lock().unwrap();

    todo!()
}

/// `RawEvent` with `PartialOrd` and `Ord` implemented
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct OrdRawEvent (RawEvent);

impl Deref for OrdRawEvent {
    type Target = RawEvent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialOrd for OrdRawEvent {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.id().partial_cmp(&other.0.id())
    }
}

impl Ord for OrdRawEvent {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.id().cmp(&other.0.id())
    }
}