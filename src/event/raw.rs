use crate::core::*;
use std::ffi::c_void;
use std::{mem::MaybeUninit, ptr::{addr_of, NonNull}};
use opencl_sys::{cl_event, clRetainEvent, clReleaseEvent, clGetEventInfo, cl_event_info, clWaitForEvents};
use rscl_proc::docfg;
use super::{Event};

/// A raw OpenCL event
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

    /// Blocks the current thread until the event has completed, without consuming it.
    #[inline(always)]
    pub fn wait_by_ref (&self) -> Result<()> {
        unsafe {
            tri!(clWaitForEvents(1, self as *const _ as *const _))
        }

        Ok(())
    }

    /// Blocks the current thread until all the events have completed, consuming them.
    #[inline(always)]
    pub fn wait_all (v: &[RawEvent]) -> Result<()> {
        let len = u32::try_from(v.len()).unwrap();

        unsafe {
            tri!(clWaitForEvents(len, v.as_ptr().cast()))
        }

        Ok(())
    }

    #[inline(always)]
    pub(super) fn get_info<T> (&self, id: cl_event_info) -> Result<T> {
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
    /// Adds a callback function that will be executed when the event is submitted
    #[inline(always)]
    pub fn on_submit (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Submitted, f)
    }

    /// Adds a callback function that will be executed when the event starts running
    #[inline(always)]
    pub fn on_run (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Running, f)
    }

    /// Adds a callback function that will be executed when the event completes
    #[inline(always)]
    pub fn on_complete (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Complete, f)
    }

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
            self.on_status_raw(status, event_listener, user_data.cast())
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
        tri!(opencl_sys::clSetEventCallback(self.id(), status as i32, Some(f), user_data));
        Ok(())
    }
}

impl Event for RawEvent {
    type Output = ();

    #[inline(always)]
    fn consume (self) -> Self::Output {
        ()
    }

    #[inline(always)]
    fn wait (self) -> Result<()> {
        unsafe {
            tri!(clWaitForEvents(1, addr_of!(self).cast()))
        }

        Ok(())
    }
}

impl AsRef<RawEvent> for RawEvent {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        self
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
    let f = *Box::from_raw(user_data);
    
    let event = RawEvent::from_id_unchecked(event);
    let status = EventStatus::try_from(event_command_status);
    f(event, status)
}