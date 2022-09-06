use crate::core::*;
use std::ffi::c_void;
use std::time::{Duration, SystemTime};
use std::{mem::MaybeUninit, ptr::{NonNull}};
use opencl_sys::*;
use blaze_proc::docfg;
use super::{EventStatus, ProfilingInfo, CommandType, Event};

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
        self.get_info(CL_EVENT_COMMAND_QUEUE).map(RawCommandQueue::from_id)
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

impl<'a> Into<Event<'a, ()>> for RawEvent {
    #[inline(always)]
    fn into(self) -> Event<'a, ()> {
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