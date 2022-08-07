use std::ffi::c_void;
use opencl_sys::*;
use crate::{prelude::{RawEvent, Result}, event::EventStatus};

pub(super) enum Listener {
    Raw (unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), *mut c_void),
    Boxed (Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>),
}

impl Listener {
    #[inline]
    pub fn call (self, evt: &RawEvent, status: Result<EventStatus>) {
        match self {
            Self::Raw(f, user_data) => unsafe {
                evt.retain().unwrap();
                let status = match status {
                    Ok(x) => x as cl_int,
                    Err(e) => e.ty as cl_int
                };

                f(evt.id(), status, user_data);
            },

            Self::Boxed(f) => f(evt.clone(), status)
        }
    }
}

unsafe impl Send for Listener {}
unsafe impl Sync for Listener {}