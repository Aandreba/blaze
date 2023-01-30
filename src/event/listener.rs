use super::{EventStatus, RawEvent};
use crate::prelude::*;
use once_cell::unsync::OnceCell;
use opencl_sys::*;
use std::{ffi::c_void, panic::*, sync::Arc};
use utils_atomics::FillQueue;

thread_local! {
    static LISTENER_THREAD_STATUS : OnceCell<Arc<FillQueue<EventCallback>>> = OnceCell::new();
}

pub(super) fn get_sender () -> Arc<FillQueue<EventCallback>> {
    LISTENER_THREAD_STATUS.with(|lts| {
        lts.get_or_init(|| {
            let queue = Arc::new(FillQueue::new());
            let recv = Arc::downgrade(&queue);

            std::thread::Builder::new()
                .name(String::from("Callback Handler"))
                .spawn(move || {
                    let mut callbacks = Vec::<EventCallback>::with_capacity(1);
                    let mut recv = Some(recv);

                    loop {
                        if let Some(ref queue) = recv {
                            match queue.upgrade() {
                                Some(queue) => callbacks.extend(queue.chop()),
                                None => recv = None
                            }
                        }

                        if recv.is_none() && callbacks.is_empty() {
                            break
                        }

                        let mut i = 0;
                        while i < callbacks.len() {
                            let callback = unsafe { callbacks.get_unchecked(i) };
                            let status = match callback.evt.status() {
                                Ok(s) if s <= callback.status => Ok(callback.status),
                                e @ Err(_) => e,
                                Ok(_) => {
                                    i += 1;
                                    continue;
                                }
                            };

                            let callback = callbacks.swap_remove(i);
                            let v = match callback.cb {
                                Callback::Boxed(f) => {
                                    catch_unwind(AssertUnwindSafe(|| f(callback.evt, status)))
                                }
                                Callback::Raw(f, user_data) => unsafe {
                                    let status = match status {
                                        Ok(x) => x as i32,
                                        Err(e) => e.ty.as_i32(),
                                    };

                                    catch_unwind(AssertUnwindSafe(|| {
                                        f(callback.evt.id(), status, user_data)
                                    }))
                                },
                            };

                            if let Err(e) = v {
                                #[cfg(debug_assertions)]
                                resume_unwind(e);
                                #[cfg(not(debug_assertions))]
                                match Box::<dyn 'static + std::any::Any + Send>::downcast::<String>(e) {
                                    Ok(x) => eprintln!("{x}"),
                                    Err(e) => match Box::<dyn 'static + std::any::Any + Send>::downcast::<&'static str>(e) {
                                        Ok(x) => eprintln!("{x}"),
                                        Err(e) => eprintln!("Panic with non-strig payload")
                                    }
                                }
                            }
                        }
                    }
                })
                .unwrap();
            return queue
        }).clone()
    })
}

pub(super) struct EventCallback {
    pub evt: RawEvent,
    pub status: EventStatus,
    pub cb: Callback,
}

pub(super) enum Callback {
    Boxed(Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send + Sync>),
    Raw(
        unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void),
        *mut c_void,
    ),
}

unsafe impl Send for Callback {}
unsafe impl Sync for Callback {}
