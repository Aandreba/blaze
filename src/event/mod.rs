use std::{mem::ManuallyDrop};
use opencl_sys::{CL_COMMAND_NDRANGE_KERNEL, CL_COMMAND_TASK, CL_COMMAND_NATIVE_KERNEL, CL_COMMAND_READ_BUFFER, CL_COMMAND_WRITE_BUFFER, CL_COMMAND_COPY_BUFFER, CL_COMMAND_READ_IMAGE, CL_COMMAND_WRITE_IMAGE, CL_COMMAND_COPY_IMAGE, CL_COMMAND_COPY_IMAGE_TO_BUFFER, CL_COMMAND_COPY_BUFFER_TO_IMAGE, CL_COMMAND_MAP_BUFFER, CL_COMMAND_MAP_IMAGE, CL_COMMAND_UNMAP_MEM_OBJECT, CL_COMMAND_MARKER, CL_COMMAND_ACQUIRE_GL_OBJECTS, CL_COMMAND_RELEASE_GL_OBJECTS, CL_EVENT_COMMAND_TYPE, CL_EVENT_COMMAND_EXECUTION_STATUS, cl_command_queue, CL_EVENT_COMMAND_QUEUE, cl_event};
use crate::core::*;

flat_mod!(status, raw);

#[cfg(feature = "cl1_1")]
flat_mod!(flag);

#[cfg(feature = "futures")]
flat_mod!(wait);

pub trait Event: AsRef<RawEvent> {
    type Output;

    fn consume (self) -> Self::Output;

    #[inline(always)]
    fn raw (&self) -> RawEvent {
        self.as_ref().clone()
    }

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

#[derive(Clone)]
#[repr(transparent)]
pub struct WaitList (pub(crate) Option<Vec<RawEvent>>);

impl WaitList {
    pub const EMPTY : Self = WaitList(None);

    #[inline(always)]
    pub fn new (wait: Vec<RawEvent>) -> Self {
        if wait.len() == 0 {
            return Self::EMPTY;
        }

        Self(Some(wait))
    }

    #[inline(always)]
    pub fn from_array<const N: usize> (wait: [RawEvent;N]) -> Self {
        Self(Some(wait.to_vec()))
    }

    #[inline(always)]
    pub fn from_event (wait: RawEvent) -> Self {
        Self::from_array([wait])
    }

    #[inline(always)]
    pub fn push (&mut self, evt: RawEvent) {
        match self.0 {
            Some(ref mut x) => x.push(evt),
            None => self.0 = Some(vec![evt])
        }
    }

    #[inline(always)]
    pub fn raw_parts (&self) -> (u32, *const cl_event) {
        match self.0 {
            Some(ref x) => {
                (u32::try_from(x.len()).unwrap(), x.as_ptr().cast())
            },
            
            None => (0, core::ptr::null())
        }
    }

    pub fn extend_self (&mut self, wait: WaitList) {
        if let Some(y) = wait.0 {
            match self.0 {
                Some(ref mut x) => {
                    let y = ManuallyDrop::new(y);
                    let (y_len, y_ptr) = (y.len(), y.as_ptr());

                    unsafe {
                        x.reserve(y_len);
                        core::ptr::copy_nonoverlapping(y_ptr, x.as_mut_ptr().add(x.len()), y_len);
                        x.set_len(x.len() + y_len)
                    }
                }

                None => self.0 = Some(y)
            }
        }
    }
}

impl Extend<RawEvent> for WaitList {
    fn extend<T: IntoIterator<Item = RawEvent>>(&mut self, iter: T) {
        let iter = iter.into_iter();

        match self.0 {
            Some(ref mut x) => x.extend(iter),
            None => *self = Self::new(iter.collect::<Vec<_>>())
        }
    }

    #[inline(always)]
    fn extend_one(&mut self, item: RawEvent) {
        self.push(item)
    }
}

impl<T: AsRef<RawEvent>> From<T> for WaitList {
    #[inline(always)]
    fn from(x: T) -> Self {
        Self::from_array([x.as_ref().clone()])
    }
}

impl From<Vec<RawEvent>> for WaitList {
    #[inline(always)]
    fn from(x: Vec<RawEvent>) -> Self {
        Self::new(x)
    }
}

impl<const N: usize> From<[RawEvent; N]> for WaitList {
    #[inline(always)]
    fn from(x: [RawEvent; N]) -> Self {
        Self::from_array(x)
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