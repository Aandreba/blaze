use std::{ptr::addr_of_mut, time::{Duration, SystemTime}};
use opencl_sys::*;
use super::RawEvent;
use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ProfilingInfo<T> {
    /// Value that describes the current device time counter in nanoseconds when the command identified by event is enqueued in a command-queue by the host.
    pub queued: T,
    /// Value that describes the current device time counter in nanoseconds when the command identified by event that has been enqueued is submitted by the host to the device associated with the command-queue.
    pub submit: T,
    /// Value that describes the current device time counter in nanoseconds when the command identified by event starts execution on the device.
    pub start: T,
    /// Value that describes the current device time counter in nanoseconds when the command identified by event has finished execution on the device.
    pub end: T,
    /// Value that describes the current device time counter in nanoseconds when the command identified by event and any child commands enqueued by this command on the device have finished execution.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    pub complete: T
}

impl ProfilingInfo<u64> {
    #[inline]
    pub fn new (event: &RawEvent) -> Result<Self> {
        let queued = Self::get_info(event, CL_PROFILING_COMMAND_QUEUED)?;
        let submit = Self::get_info(event, CL_PROFILING_COMMAND_SUBMIT)?;
        let start = Self::get_info(event, CL_PROFILING_COMMAND_START)?;
        let end = Self::get_info(event, CL_PROFILING_COMMAND_END)?;
        #[cfg(feature = "cl2")]
        let complete = Self::get_info(event, CL_PROFILING_COMMAND_COMPLETE)?;
        
        Ok(Self { queued, submit, start, end, #[cfg(feature = "cl2")] complete  })
    }

    /// Time elapsed between [`ProfilingInfo::start`] and [`ProfilingInfo::end`]
    #[inline(always)]
    pub fn duration (&self) -> Duration {
        Duration::from_nanos(self.end - self.start)
    }

    #[inline(always)]
    fn get_info (event: &RawEvent, ty: cl_profiling_info) -> Result<u64> {
        let mut value = 0u64;
        unsafe {
            tri!(clGetEventProfilingInfo(event.id(), ty, core::mem::size_of::<u64>(), addr_of_mut!(value).cast(), core::ptr::null_mut()))
        }

        Ok(value)
    }
}

impl ProfilingInfo<SystemTime> {
    #[inline]
    pub fn new (event: &RawEvent) -> Result<Self> {
        let queued = Self::get_info(event, CL_PROFILING_COMMAND_QUEUED)?;
        let submit = Self::get_info(event, CL_PROFILING_COMMAND_SUBMIT)?;
        let start = Self::get_info(event, CL_PROFILING_COMMAND_START)?;
        let end = Self::get_info(event, CL_PROFILING_COMMAND_END)?;
        #[cfg(feature = "cl2")]
        let complete = Self::get_info(event, CL_PROFILING_COMMAND_COMPLETE)?;
        
        Ok(Self { queued, submit, start, end, #[cfg(feature = "cl2")] complete  })
    }

    /// Time elapsed between [`ProfilingInfo::start`] and [`ProfilingInfo::end`]
    #[inline(always)]
    pub fn duration (&self) -> Duration {
        self.end.duration_since(self.start).unwrap()
    }

    #[inline(always)]
    fn get_info (event: &RawEvent, ty: cl_profiling_info) -> Result<SystemTime> {
        let nanos = ProfilingInfo::<u64>::get_info(event, ty)?;
        Ok(std::time::UNIX_EPOCH + Duration::from_nanos(nanos))
    }
}