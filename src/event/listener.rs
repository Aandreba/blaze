use crate::prelude::*;
use super::{RawEvent, EventStatus};
use std::{sync::{mpsc::{Receiver, Sender, channel, TryRecvError}}, ffi::c_void, panic::{catch_unwind, AssertUnwindSafe}};
use once_cell::unsync::OnceCell;
use opencl_sys::*;

type ListenerSender = Sender<(RawEvent, Receiver<EventCallback>)>;

thread_local! {
    static LISTENER_THREAD_STATUS : OnceCell<ListenerSender> = OnceCell::new();
}

pub(super) fn get_sender () -> ListenerSender {
    LISTENER_THREAD_STATUS.with(|lts| {
        lts.get_or_init(|| {
            let (send, recv) = channel();
            
            std::thread::spawn(move || {
                let mut listeners = Vec::<Listener>::new();
                let mut connected = true;

                while connected || !listeners.is_empty() {
                    // Check for new listeners to add to the queue
                    if connected {
                        loop {
                            match recv.try_recv() {
                                Ok((evt, recv)) => listeners.push(Listener { evt, recv, cbs: Vec::new(), closed: false }),
                                Err(TryRecvError::Empty) => break,
                                Err(TryRecvError::Disconnected) => connected = false
                            }
                        }
                    }

                    // Check for callbacks to add to the list
                    for listener in listeners.iter_mut().filter(|x| !x.closed) {
                        loop {
                            match listener.recv.try_recv() {
                                Ok(cb) => listener.cbs.push(cb),
                                Err(TryRecvError::Disconnected) => listener.closed = true,
                                Err(TryRecvError::Empty) => break,
                            }
                        }
                    }

                    // Consume the appropiate listeners
                    let mut i = 0;
                    while i < listeners.len() {
                        let listener = unsafe { listeners.get_unchecked_mut(i) };
                        let cbs = &mut listener.cbs;

                        let status = listener.evt.status();
                        let status_num = match &status {
                            Ok(status) => *status as i32,
                            Err(e) => e.ty as i32 
                        };

                        // Consume appropiate listener's callbacks
                        let mut j = 0;
                        while j < cbs.len() {
                            let cb = unsafe { cbs.get_unchecked(j) };
                            
                            if status_num <= cb.status as i32 {
                                match cbs.swap_remove(j).cb {
                                    Callback::Boxed(f) => {
                                        let _ = catch_unwind(AssertUnwindSafe(|| f(listener.evt.clone(), status.clone())));
                                    },

                                    Callback::Raw(f, user_data) => unsafe {
                                        let f = AssertUnwindSafe(f);
                                        let _ = catch_unwind(|| f(listener.evt.id(), status_num, user_data));
                                    }
                                }
                                continue;
                            }

                            j += 1;
                        }

                        // If channel closed and no more callbacks exist, remove listener 
                        if listener.closed && cbs.len() == 0 {
                            listeners.swap_remove(i);
                            continue;
                        }

                        i += 1;
                    }
                }
            });

            return send
        }).clone()
    })
}

struct Listener {
    evt: RawEvent,
    cbs: Vec<EventCallback>,
    recv: Receiver<EventCallback>,
    closed: bool
}

pub(super) struct EventCallback {
    pub status: EventStatus,
    pub cb: Callback
}

pub(super) enum Callback {
    Boxed (Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>),
    Raw (unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), *mut c_void)
}

unsafe impl Send for Callback {}
unsafe impl Sync for Callback {}