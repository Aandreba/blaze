use crate::core::*;
use std::ffi::c_void;
use std::{mem::MaybeUninit, ptr::{addr_of, NonNull}};
use opencl_sys::{cl_event, clRetainEvent, clReleaseEvent, clGetEventInfo, cl_event_info, clWaitForEvents};
use blaze_proc::docfg;
use super::{Event};

/// A raw OpenCL event
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawEvent (NonNull<c_void>);

impl RawEvent {
    #[inline(always)]
    pub const unsafe fn from_id_unchecked (inner: cl_event) -> Self {
        Self(NonNull::new_unchecked(inner))
    }

    #[inline(always)]
    pub const fn from_id (inner: cl_event) -> Option<Self> {
        NonNull::new(inner).map(Self)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_event {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub unsafe fn retain (&self) -> Result<()> {
        tri!(clRetainEvent(self.id()));
        Ok(())
    }

    /// Blocks the current thread until all the events have completed
    #[inline(always)]
    pub fn wait_all (v: &[RawEvent]) -> Result<()> {
        let len = u32::try_from(v.len()).unwrap();

        unsafe {
            tri!(clWaitForEvents(len, v.as_ptr().cast()))
        }

        Ok(())
    }

    #[inline(always)]
    pub(super) fn get_info<T: Copy> (&self, id: cl_event_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetEventInfo(self.id(), id, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

#[cfg(feature = "cl1_1")]
use {opencl_sys::cl_int, super::EventStatus};

#[docfg(feature = "cl1_1")]
impl RawEvent {
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
        let user_data = Box::into_raw(Box::new(f));
        
        unsafe {
            if let Err(e) = self.on_status_raw(status, event_listener, user_data.cast()) {
                let _ = Box::from_raw(user_data);
                return Err(e)
            }

            tri!(clRetainEvent(self.id()));
        }

        Ok(())
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
        tri!(opencl_sys::clSetEventCallback(self.id(), status as i32, Some(f), user_data));
        Ok(())
    }
}

impl Event for RawEvent {
    type Output = ();

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent { self }

    #[inline(always)]
    fn consume (self, error: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = error { return Err(err); }
        Ok(())
    }

    #[inline(always)]
    fn wait (self) -> Result<()> {
        unsafe {
            tri!(clWaitForEvents(1, addr_of!(self).cast()))
        }

        Ok(())
    }

    #[inline(always)]
    fn wait_by_ref (&self) -> Result<()> {
        unsafe {
            tri!(clWaitForEvents(1, self as *const _ as *const _))
        }

        Ok(())
    }
}

impl Event for &RawEvent {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        Ok(())
    }

    #[inline(always)]
    fn wait (self) -> Result<()> {
        unsafe {
            tri!(clWaitForEvents(1, addr_of!(self).cast()))
        }

        Ok(())
    }
}

impl Clone for RawEvent {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainEvent(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for RawEvent {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseEvent(self.id()))
        }
    }
}

unsafe impl Send for RawEvent {}
unsafe impl Sync for RawEvent {}

#[cfg(feature = "cl1_1")]
unsafe extern "C" fn event_listener (event: cl_event, event_command_status: cl_int, user_data: *mut c_void) {
    let user_data : *mut Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send> = user_data.cast();
    let f = Box::into_inner(Box::from_raw(user_data));
    
    let event = RawEvent::from_id_unchecked(event);
    let status = EventStatus::try_from(event_command_status);
    f(event, status)
}