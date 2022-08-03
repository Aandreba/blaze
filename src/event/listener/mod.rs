#![allow(unused)]

use std::{collections::{HashMap}, ffi::c_void, sync::{Mutex}, ops::{Deref, DerefMut}};
use crate::prelude::*;
use super::EventStatus;
use once_cell::sync::Lazy;
use opencl_sys::*;

flat_mod!(ty, thread, queue);

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

static SUBMITTING : Mutex<ListenerQueue> = Mutex::new(ListenerQueue::new());
static RUNNING : Mutex<ListenerQueue> = Mutex::new(ListenerQueue::new());
static COMPLETED : Mutex<ListenerQueue> = Mutex::new(ListenerQueue::new());
static EVENTS : [&Mutex<ListenerQueue>; 3] = [&COMPLETED, &RUNNING, &SUBMITTING];

fn add_listener (evt: RawEvent, status: EventStatus, listener: Listener) -> Result<()> {
    if status.is_queued() {
        return Err(Error::new(ErrorType::InvalidValue, "Cannot listen to 'Queued' status"))
    }

    init_thread();
    let events = EVENTS[status as usize].lock().unwrap();

    todo!()
}