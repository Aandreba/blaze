use std::intrinsics::transmute;
use opencl_sys::{CL_QUEUED, CL_SUBMITTED, CL_RUNNING, CL_COMPLETE};
use crate::core::Error;

/// Status of an
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum EventStatus {
    /// Command has been enqueued in the command-queue
    Queued = CL_QUEUED,
    /// Enqueued command has been submitted by the host to the device associated with the command-queue
    Submitted = CL_SUBMITTED,
    /// Device is currently executing this command
    Running = CL_RUNNING,
    /// The command has completed 
    Complete = CL_COMPLETE
}

impl TryFrom<i32> for EventStatus {
    type Error = Error;

    #[inline(always)]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(Error::from(value))
        }

        return unsafe { Ok(transmute(value)) }
    }
}

impl Into<i32> for EventStatus {
    #[inline(always)]
    fn into(self) -> i32 {
        self as i32
    }
}