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

#[inline(always)]
pub fn add_boxed_listener (evt: &RawEvent, status: EventStatus, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
    add_listener(evt, status, Listener::Boxed(f))
}

#[inline(always)]
pub fn add_raw_listener (evt: &RawEvent, status: EventStatus, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
    add_listener(evt, status, Listener::Raw(f, user_data))
}

fn add_listener (evt: &RawEvent, status: EventStatus, listener: Listener) -> Result<()> {
    if status.is_queued() {
        return Err(Error::new(ErrorType::InvalidValue, "Cannot listen to 'Queued' status"))
    }

    init_thread();
    let mut events = match EVENTS[status as usize].lock() {
        Ok(x) => x,
        Err(e) => e.into_inner()
    };

    events.add_listener(&evt, listener);
    Ok(())
}