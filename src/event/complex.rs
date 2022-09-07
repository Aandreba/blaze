use std::{sync::mpsc::{channel, Sender}, ops::Deref, ffi::c_void, time::{SystemTime, Duration}};
use opencl_sys::*;
use blaze_proc::docfg;
use crate::prelude::*;
use super::{RawEvent, EventStatus, ProfilingInfo};

pub struct Event<'a, T> {
    inner: RawEvent,
    f: Box<dyn 'a + FnOnce() -> Result<T>>,
    #[cfg(not(feature = "cl1_1"))]
    send: Sender<super::listener::EventCallback>
}

impl<'a> Event<'a, ()> {
    #[inline(always)]
    pub(crate) fn new_noop (inner: RawEvent) -> Self {
        Self::new(inner, || Ok(()))
    }
}

impl<'a, T> Event<'a, T> {
    #[inline(always)]
    pub(crate) fn new<F: 'a + FnOnce() -> Result<T>> (inner: RawEvent, f: F) -> Self {
        Self::new_boxed(inner, Box::new(f))
    }

    pub(crate) fn new_boxed (inner: RawEvent, f: Box<dyn 'a + FnOnce() -> Result<T>>) -> Self {
        #[cfg(not(feature = "cl1_1"))]
        let (send, recv) = channel();
        #[cfg(not(feature = "cl1_1"))]
        let list = super::listener::get_sender();
        #[cfg(not(feature = "cl1_1"))]
        list.send((inner.clone(), recv)).unwrap();

        Self {
            inner,
            f,
            #[cfg(not(feature = "cl1_1"))]
            send
        }
    }

    #[inline(always)]
    pub(super) fn consume (self) -> Result<T> {
        (self.f)()
    }
}

impl<'a, T> Event<'a, T> {
    #[inline(always)]
    pub fn join (self) -> Result<T> {
        self.join_by_ref()?;
        self.consume()
    }

    #[inline]
    pub fn join_with_nanos (self) -> Result<(T, ProfilingInfo<u64>)> {
        self.join_by_ref()?;
        let nanos = self.profiling_nanos()?;
        let v = self.consume()?;
        Ok((v, nanos))
    }

    #[inline]
    pub fn join_with_time (self) -> Result<(T, ProfilingInfo<SystemTime>)> {
        self.join_by_ref()?;
        let nanos = self.profiling_time()?;
        let v = self.consume()?;
        Ok((v, nanos))
    }

    #[inline]
    pub fn join_with_duration (self) -> Result<(T, Duration)> {
        self.join_by_ref()?;
        let nanos = self.duration()?;
        let v = self.consume()?;
        Ok((v, nanos))
    }

    /// Blocks the current thread util the event has completed, returning `data` if it completed correctly, and panicking otherwise.
    #[inline(always)]
    pub fn join_unwrap (self) -> T {
        self.join().unwrap()
    }

    /// Returns a future that waits for the event to complete without blocking.
    #[inline(always)]
    #[docfg(feature = "futures")]
    pub fn join_async (self) -> Result<crate::event::EventWait<'a, T>> {
        crate::event::EventWait::new(self)
    }

    #[inline(always)]
    pub fn join_all<I: IntoIterator<Item = Self>> (iter: I) -> Result<Vec<T>> {
        todo!()
    }
}

impl<'a, T> Event<'a, T> {
    /// Adds a callback function that will be executed when the event is submitted.
    #[inline(always)]
    pub fn on_submit (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Submitted, f)
    }

    /// Adds a callback function that will be executed when the event starts running.
    #[inline(always)]
    pub fn on_run (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Running, f)
    }

    /// Adds a callback function that will be executed when the event completes.
    #[inline(always)]
    pub fn on_complete (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Complete, f)
    }

    /// Registers a user callback function for a specific command execution status.\
    /// The registered callback function will be called when the execution status of command associated with event changes to an execution status equal to or past the status specified by `status`.\
    /// Each call to [`RawEvent::on_status`] registers the specified user callback function on a callback stack associated with event. The order in which the registered user callback functions are called is undefined.\
    /// All callbacks registered for an event object must be called before the event object is destroyed. Callbacks should return promptly.\
    /// Behavior is undefined when calling expensive system routines, OpenCL APIs to create contexts or command-queues, or blocking OpenCL APIs in an event callback. Rather than calling a blocking OpenCL API in an event callback, applications may call a non-blocking OpenCL API, then register a completion callback for the non-blocking OpenCL API with the remainder of the work.\
    /// Because commands in a command-queue are not required to begin execution until the command-queue is flushed, callbacks that enqueue commands on a command-queue should either call [`RawCommandQueue::flush`] on the queue before returning, or arrange for the command-queue to be flushed later.
    #[inline(always)]
    pub fn on_status (&self, status: EventStatus, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status_boxed(status, Box::new(f) as Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>)
    }

    #[inline(always)]
    pub fn on_submit_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Submitted, f)
    }

    #[inline(always)]
    pub fn on_run_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Running, f)
    }

    #[inline(always)]
    pub fn on_complete_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Complete, f)
    }

    #[inline(always)]
    pub fn on_status_boxed (&self, status: EventStatus, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                let user_data = Box::into_raw(Box::new(f));
                unsafe {
                    if let Err(e) = self.on_status_raw(status, event_listener, user_data.cast()) {
                        let _ = Box::from_raw(user_data);
                        return Err(e);
                    }

                    tri!(clRetainEvent(self.id()));
                }

                return Ok(())
            } else {
                let cb = super::listener::EventCallback { status, cb: super::listener::Callback::Boxed(f) };
                match self.send.send(cb) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorType::InvalidValue.into())
                }
            }
        }
    }
    
    #[inline(always)]
    pub unsafe fn on_submit_raw (&self, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Submitted, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_run_raw (&self, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Running, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_complete_raw (&self, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Complete, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_status_raw (&self, status: EventStatus, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                tri!(opencl_sys::clSetEventCallback(self.id(), status as i32, Some(f), user_data));
                return Ok(())
            } else {
                let cb = super::listener::EventCallback { status, cb: super::listener::Callback::Raw(f, user_data) };
                match self.send.send(cb) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorType::InvalidValue.into())
                }
            }
        }
    }
}

impl<T> Deref for Event<'_, T> {
    type Target = RawEvent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(feature = "cl1_1")]
unsafe extern "C" fn event_listener (event: cl_event, event_command_status: cl_int, user_data: *mut c_void) {
    let user_data : *mut Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send> = user_data.cast();
    let f = *Box::from_raw(user_data);
    
    let event = RawEvent::from_id_unchecked(event);
    let status = EventStatus::try_from(event_command_status);
    f(event, status)
}