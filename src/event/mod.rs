use std::{num::NonZeroU32, intrinsics::transmute, ptr::NonNull};

use opencl_sys::{CL_COMMAND_NDRANGE_KERNEL, CL_COMMAND_TASK, CL_COMMAND_NATIVE_KERNEL, CL_COMMAND_READ_BUFFER, CL_COMMAND_WRITE_BUFFER, CL_COMMAND_COPY_BUFFER, CL_COMMAND_READ_IMAGE, CL_COMMAND_WRITE_IMAGE, CL_COMMAND_COPY_IMAGE, CL_COMMAND_COPY_IMAGE_TO_BUFFER, CL_COMMAND_COPY_BUFFER_TO_IMAGE, CL_COMMAND_MAP_BUFFER, CL_COMMAND_MAP_IMAGE, CL_COMMAND_UNMAP_MEM_OBJECT, CL_COMMAND_MARKER, CL_COMMAND_ACQUIRE_GL_OBJECTS, CL_COMMAND_RELEASE_GL_OBJECTS, CL_EVENT_COMMAND_TYPE, CL_EVENT_COMMAND_EXECUTION_STATUS, cl_command_queue, CL_EVENT_COMMAND_QUEUE, cl_event};
use crate::core::*;

flat_mod!(status, raw, flag);
#[cfg(feature = "futures")]
flat_mod!(wait);

pub trait Event: AsRef<RawEvent> {
    type Output;

    fn consume (self) -> Self::Output;

    #[inline(always)]
    fn wait (self) -> Result<Self::Output> where Self: Sized {
        self.as_ref().wait_by_ref()?;
        Ok(self.consume())
    }

    #[inline(always)]
    #[cfg(feature = "futures")]
    fn wait_async (self) -> Result<crate::event::EventWait<Self>> where Self: Sized {
        crate::event::EventWait::new(self)
    }

    #[inline(always)]
    fn ty (&self) -> Result<CommandType> {
        self.as_ref().get_info(CL_EVENT_COMMAND_TYPE)
    }

    #[inline(always)]
    fn status (&self) -> Result<EventStatus> {
        let int : i32 = self.as_ref().get_info(CL_EVENT_COMMAND_EXECUTION_STATUS)?;
        EventStatus::try_from(int)
    }

    #[inline(always)]
    fn command_queue (&self) -> Result<cl_command_queue> {
        let queue : cl_command_queue = self.as_ref().get_info(CL_EVENT_COMMAND_QUEUE)?;
        Ok(queue)
    }
}

#[repr(transparent)]
pub struct WaitList (Box<[cl_event]>);

impl WaitList {
    pub const EMPTY : Self = unsafe { transmute(NonNull::new(core::slice::from_raw_parts_mut::<cl_event>(core::ptr::null_mut(), 0)).unwrap()) };

    #[inline(always)]
    pub fn from_iter (wait: impl IntoIterator<Item = impl AsRef<RawEvent>>) -> Self {
        let wait = wait.into_iter().map(|x| x.as_ref().0).collect::<Box<[_]>>();
        Self::from_boxed_slice(wait)
    }

    #[inline(always)]
    pub fn from_raw (wait: impl Into<Box<[RawEvent]>>) -> Self {
        Self::from_boxed_slice(unsafe { transmute(wait.into()) })
    }

    #[inline(always)]
    pub fn from_boxed_slice (wait: Box<[cl_event]>) -> Self {
        Self(wait)
    }

    #[inline(always)]
    pub fn raw_parts (&self) -> (u32, *const cl_event) {
        (u32::try_from(self.0.len()).unwrap(), self.0.as_ptr())
    }
}

impl<I: IntoIterator> From<I> for WaitList where I::Item: AsRef<RawEvent> {
    #[inline(always)]
    fn from(wait: I) -> Self {
        Self::from_iter(wait)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum CommandType {
    NdRangeKernel = CL_COMMAND_NDRANGE_KERNEL,
    Task = CL_COMMAND_TASK,
    NativeKernel = CL_COMMAND_NATIVE_KERNEL,
    ReadBuffer = CL_COMMAND_READ_BUFFER,
    WriteBuffer = CL_COMMAND_WRITE_BUFFER,
    CopyBuffer = CL_COMMAND_COPY_BUFFER,
    ReadImage = CL_COMMAND_READ_IMAGE,
    WriteImage = CL_COMMAND_WRITE_IMAGE,
    CopyImage = CL_COMMAND_COPY_IMAGE,
    CopyImageToBuffer = CL_COMMAND_COPY_IMAGE_TO_BUFFER,
    CopyBufferToImage = CL_COMMAND_COPY_BUFFER_TO_IMAGE,
    MapBuffer = CL_COMMAND_MAP_BUFFER,
    MapImage = CL_COMMAND_MAP_IMAGE,
    UnmapMemObject = CL_COMMAND_UNMAP_MEM_OBJECT,
    Marker = CL_COMMAND_MARKER,
    AcquireGLObjects = CL_COMMAND_ACQUIRE_GL_OBJECTS,
    ReleaseGLObjects = CL_COMMAND_RELEASE_GL_OBJECTS
}