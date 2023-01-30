use crate::prelude::*;
use super::{RawEvent, EventStatus};
use std::{panic::*, sync::{mpsc::{Sender, channel, TryRecvError}}, ffi::c_void};
use once_cell::unsync::OnceCell;
use opencl_sys::*;

thread_local! {
    static LISTENER_THREAD_STATUS : OnceCell<Sender<EventCallback>> = OnceCell::new();
}

pub(super) fn get_sender () -> Sender<EventCallback> {
    LISTENER_THREAD_STATUS.with(|lts| {
        lts.get_or_init(|| {
            let (send, recv) = channel::<EventCallback>();
            
            std::thread::Builder::new()
                .name(String::from("Callback Handler"))
                .spawn(move || {
                    let mut callbacks = Vec::<EventCallback>::with_capacity(1);
                    let mut open = true;

                    while open {
                        loop {
                            match recv.try_recv() {
                                Ok(x) => callbacks.push(x),
                                Err(TryRecvError::Disconnected) => break open = false,
                                Err(TryRecvError::Empty) => break
                            }
                        };

                        let mut i = 0;
                        while i < callbacks.len() {
                            let callback = unsafe { callbacks.get_unchecked(i) };
                            let status = match callback.evt.status() {
                                Ok(s) if s <= callback.status => Ok(callback.status),
                                e @ Err(_) => e,
                                Ok(_) => {
                                    i += 1;
                                    continue
                                }
                            };

                            let callback = callbacks.swap_remove(i);
                            let v = match callback.cb {
                                Callback::Boxed(f) => catch_unwind(AssertUnwindSafe(|| f(callback.evt, status))),
                                Callback::Raw(f, user_data) => unsafe {
                                    let status = match status {
                                        Ok(x) => x as i32,
                                        Err(e) => e.ty.as_i32()
                                    };
                                    
                                    catch_unwind(AssertUnwindSafe(|| f(callback.evt.id(), status, user_data)))
                                }
                            };

                            if let Err(e) = v {
                                #[cfg(debug_assertions)]
                                resume_unwind(e);
                                #[cfg(not(debug_assertions))]
                                todo!()
                            }
                        }
                    }
                    
                }).unwrap();

            return send
        }).clone()
    })
}

pub(super) struct EventCallback {
    pub evt: RawEvent,
    pub status: EventStatus,
    pub cb: Callback
}

pub(super) enum Callback {
    Boxed (Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>),
    Raw (unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), *mut c_void)
}

unsafe impl Send for Callback {}
unsafe impl Sync for Callback {}