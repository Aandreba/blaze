use crate::core::*;
use std::ffi::c_void;
use std::time::{Duration, SystemTime};
use std::{mem::MaybeUninit, ptr::{NonNull}};
use opencl_sys::*;
use blaze_proc::docfg;
use super::ext::NoopEvent;
use super::{EventStatus, ProfilingInfo, CommandType, Event};

/// Raw OpenCL event
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawEvent (NonNull<c_void>);

impl RawEvent {
    #[inline(always)]
    pub const unsafe fn from_id_unchecked (inner: cl_event) -> Self {
        Self(NonNull::new_unchecked(inner))
    }

    #[inline(always)]
    pub const unsafe fn from_id (inner: cl_event) -> Option<Self> {
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

    #[inline(always)]
    pub fn join_by_ref (&self) -> Result<()> {
        let slice = &[self.0.as_ptr()];

        unsafe {
            tri!(clWaitForEvents(1, slice.as_ptr()))
        }

        Ok(())
    }

    /// Blocks the current thread until all the events have completed
    #[inline(always)]
    pub fn join_all_by_ref (v: &[RawEvent]) -> Result<()> {
        let len = u32::try_from(v.len()).unwrap();

        unsafe {
            tri!(clWaitForEvents(len, v.as_ptr().cast()))
        }

        Ok(())
    }
}

impl RawEvent {
    /// Converts the [`RawEvent`] into a [`NoopEvent`].
    #[inline(always)]
    pub fn into_event (self) -> NoopEvent {
        self.into()
    }

    #[inline(always)]
    pub fn join_with_nanos_by_ref (self) -> Result<ProfilingInfo<u64>> {
        self.join_by_ref()?;
        self.profiling_nanos()
    }

    #[inline(always)]
    pub fn join_with_time_by_ref (self) -> Result<ProfilingInfo<SystemTime>> {
        self.join_by_ref()?;
        self.profiling_time()
    }

    #[inline(always)]
    pub fn join_with_duration_by_ref (self) -> Result<Duration> {
        self.join_by_ref()?;
        self.duration()
    }

    /// Blocks the current thread util the event has completed, returning `data` if it completed correctly, and panicking otherwise.
    #[inline(always)]
    pub fn join_unwrap_by_ref (self) {
        self.join_by_ref().unwrap()
    }

    /// Returns the event's type
    #[inline(always)]
    pub fn ty (&self) -> Result<CommandType> {
        self.get_info(CL_EVENT_COMMAND_TYPE)
    }

    /// Returns the event's current status
    #[inline(always)]
    pub fn status (&self) -> Result<EventStatus> {
        let int : i32 = self.get_info(CL_EVENT_COMMAND_EXECUTION_STATUS)?;
        EventStatus::try_from(int)
    }

    /// Returns the event's underlying command queue
    #[inline(always)]
    pub fn command_queue (&self) -> Result<Option<RawCommandQueue>> {
        match self.get_info(CL_EVENT_COMMAND_QUEUE) {
            Ok(x) => unsafe { Ok(RawCommandQueue::from_id(x)) },
            Err(e) => Err(e)
        }
    }

    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(opencl_sys::CL_EVENT_REFERENCE_COUNT)
    }

    /// Return the context associated with event.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn raw_context (&self) -> Result<crate::prelude::RawContext> {
        let ctx = self.get_info::<cl_context>(CL_EVENT_CONTEXT)?;
        unsafe { 
            tri!(clRetainContext(ctx));
            // SAFETY: Context checked to be valid by `clRetainContext`.
            Ok(crate::prelude::RawContext::from_id_unchecked(ctx))
        }
    }

    /// Returns this event's profiling info in `u64` nanoseconds.
    #[inline(always)]
    pub fn profiling_nanos (&self) -> Result<ProfilingInfo<u64>> {
        ProfilingInfo::<u64>::new(self)
    }

    /// Returns this event's profiling info in [`SystemTime`].
    #[inline(always)]
    pub fn profiling_time (&self) -> Result<ProfilingInfo<SystemTime>> {
        ProfilingInfo::<SystemTime>::new(self)
    }

    /// Returns the time elapsed between the event's start and end.
    #[inline(always)]
    pub fn duration (&self) -> Result<Duration> {
        let nanos = self.profiling_nanos()?;
        Ok(nanos.duration())
    }

    /// Returns `true` if the status of the event is [`EventStatus::Queued`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn is_queued (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::is_queued)
    }

    /// Returns `true` if the status of the event is [`EventStatus::Submitted`], [`EventStatus::Running`], [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn has_submited (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_submitted)
    }

    /// Returns `true` if the status of the event is [`EventStatus::Running`], [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn has_started_running (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_started_running)
    }
    
    /// Returns `true` if the status of the event is [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn has_completed (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_completed)
    }
    
    #[inline(always)]
    pub fn get_info<T: Copy> (&self, id: cl_event_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetEventInfo(self.id(), id, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

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
    /// Each call to [`Event::on_status`] registers the specified user callback function on a callback stack associated with event. The order in which the registered user callback functions are called is undefined.\
    /// All callbacks registered for an event object must be called before the event object is destroyed. Callbacks should return promptly.\
    /// Behavior is undefined when calling expensive system routines, OpenCL APIs to create contexts or command-queues, or blocking OpenCL APIs in an event callback. Rather than calling a blocking OpenCL API in an event callback, applications may call a non-blocking OpenCL API, then register a completion callback for the non-blocking OpenCL API with the remainder of the work.\
    /// Because commands in a command-queue are not required to begin execution until the command-queue is flushed, callbacks that enqueue commands on a command-queue should either call [`RawCommandQueue::flush`] on the queue before returning, or arrange for the command-queue to be flushed later.
    #[inline(always)]
    pub fn on_status (&self, status: EventStatus, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status_boxed(status, Box::new(f))
    }

    /// Adds a callback function that will be executed when the event is submitted.
    #[inline(always)]
    pub fn on_submit_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Submitted, f)
    }

    /// Adds a callback function that will be executed when the event starts running.
    #[inline(always)]
    pub fn on_run_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Running, f)
    }

    /// Adds a callback function that will be executed when the event completes.
    #[inline(always)]
    pub fn on_complete_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Complete, f)
    }

    #[inline(always)]
    pub fn on_status_boxed (&self, status: EventStatus, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        let user_data = Box::into_raw(Box::new(f));
        unsafe {
            if let Err(e) = self.on_status_raw(status, event_listener, user_data.cast()) {
                let _ = Box::from_raw(user_data); // drop user data
                return Err(e);
            }

            tri!(clRetainEvent(self.id()));
        }

        return Ok(())
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
        return Ok(())
    }
}

impl Into<NoopEvent> for RawEvent {
    #[inline(always)]
    fn into(self) -> NoopEvent {
        Event::new_noop(self)
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