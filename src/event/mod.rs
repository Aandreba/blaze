flat_mod!(raw, complex, status, profiling, eventual);

#[path = "consumer.rs"]
mod _consumer;

pub mod consumer {
    pub use super::_consumer::*;
    pub use super::complex::ext::*;
    #[cfg(feature = "cl1_1")]
    pub use super::abort::Abort;
}

pub use consumer::{Consumer, IncompleteConsumer};

#[cfg(feature = "cl1_1")]
flat_mod!(flag);

#[cfg(feature = "cl1_1")]
mod abort;
#[cfg(feature = "cl1_1")]
pub use abort::AbortHandle;

//#[cfg(not(feature = "cl1_1"))]
mod listener;

#[cfg(feature = "futures")]
flat_mod!(wait);

use opencl_sys::*;

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