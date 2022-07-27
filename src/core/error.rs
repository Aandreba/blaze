use std::{backtrace::Backtrace, sync::Arc, fmt::{Display, Debug}};
use blaze_proc::error;
pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Clone)]
#[non_exhaustive]
pub struct Error {
    pub ty: ErrorType,
    pub desc: Option<String>,
    #[cfg(debug_assertions)]
    pub backtrace: Arc<Backtrace>
}

impl Error {
    #[inline(always)]
    pub fn new (ty: ErrorType, desc: impl ToString) -> Self {
        Self::from_parts(ty, Some(desc.to_string()), Arc::new(Backtrace::capture()))
    }

    #[inline(always)]
    pub fn from_parts (ty: ErrorType, desc: Option<String>, #[cfg(debug_assertions)] backtrace: Arc<Backtrace>) -> Self {
        Self { 
            ty,
            desc: desc,
            #[cfg(debug_assertions)]
            backtrace
        }
    }

    #[inline(always)]
    pub fn from_type (ty: ErrorType) -> Self {
        Self { 
            ty,
            desc: None,
            #[cfg(debug_assertions)]
            backtrace: Arc::new(Backtrace::capture())
        }
    }
}

impl From<ErrorType> for Error {
    #[inline(always)]
    fn from(ty: ErrorType) -> Self {
        Self::from_type(ty)
    }
}

impl From<i32> for Error {
    #[inline(always)]
    fn from(x: i32) -> Self {
        Self::from_type(ErrorType::from(x))
    }
}

impl Debug for Error {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.ty, f)?;
        
        if let Some(ref desc) = self.desc {
            write!(f, ": {desc}")?;
        }

        #[cfg(debug_assertions)]
        write!(f, "\n{}", self.backtrace)?;

        Ok(())
    }
}

error! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ErrorType {
        const CL_DEVICE_NOT_FOUND,
        const CL_DEVICE_NOT_AVAILABLE,
        const CL_COMPILER_NOT_AVAILABLE,
        const CL_MEM_OBJECT_ALLOCATION_FAILURE,
        const CL_OUT_OF_RESOURCES,
        const CL_OUT_OF_HOST_MEMORY,
        const CL_PROFILING_INFO_NOT_AVAILABLE,
        const CL_MEM_COPY_OVERLAP,
        const CL_IMAGE_FORMAT_MISMATCH,
        const CL_IMAGE_FORMAT_NOT_SUPPORTED,
        const CL_BUILD_PROGRAM_FAILURE,
        const CL_MAP_FAILURE,

        // #ifdef CL_VERSION_1_1
        const CL_MISALIGNED_SUB_BUFFER_OFFSET,
        const CL_EXEC_STATUS_ERROR_FOR_EVENTS_IN_WAIT_LIST,
        // #endif

        // #ifdef CL_VERSION_1_2
        const CL_COMPILE_PROGRAM_FAILURE,
        const CL_LINKER_NOT_AVAILABLE,
        const CL_LINK_PROGRAM_FAILURE,
        const CL_DEVICE_PARTITION_FAILED,
        const CL_KERNEL_ARG_INFO_NOT_AVAILABLE,
        // #endif

        const CL_INVALID_VALUE,
        const CL_INVALID_DEVICE_TYPE,
        const CL_INVALID_PLATFORM,
        const CL_INVALID_DEVICE,
        const CL_INVALID_CONTEXT,
        const CL_INVALID_QUEUE_PROPERTIES,
        const CL_INVALID_COMMAND_QUEUE,
        const CL_INVALID_HOST_PTR,
        const CL_INVALID_MEM_OBJECT,
        const CL_INVALID_IMAGE_FORMAT_DESCRIPTOR,
        const CL_INVALID_IMAGE_SIZE,
        const CL_INVALID_SAMPLER,
        const CL_INVALID_BINARY,
        const CL_INVALID_BUILD_OPTIONS,
        const CL_INVALID_PROGRAM,
        const CL_INVALID_PROGRAM_EXECUTABLE,
        const CL_INVALID_KERNEL_NAME,
        const CL_INVALID_KERNEL_DEFINITION,
        const CL_INVALID_KERNEL,
        const CL_INVALID_ARG_INDEX,
        const CL_INVALID_ARG_VALUE,
        const CL_INVALID_ARG_SIZE,
        const CL_INVALID_KERNEL_ARGS,
        const CL_INVALID_WORK_DIMENSION,
        const CL_INVALID_WORK_GROUP_SIZE,
        const CL_INVALID_WORK_ITEM_SIZE,
        const CL_INVALID_GLOBAL_OFFSET,
        const CL_INVALID_EVENT_WAIT_LIST,
        const CL_INVALID_EVENT,
        const CL_INVALID_OPERATION,
        const CL_INVALID_GL_OBJECT,
        const CL_INVALID_BUFFER_SIZE,
        const CL_INVALID_MIP_LEVEL,
        const CL_INVALID_GLOBAL_WORK_SIZE,
        
        // #ifdef CL_VERSION_1_1
        const CL_INVALID_PROPERTY,
        // #endif
        
        // #ifdef CL_VERSION_1_2
        const CL_INVALID_IMAGE_DESCRIPTOR,
        const CL_INVALID_COMPILER_OPTIONS,
        const CL_INVALID_LINKER_OPTIONS,
        const CL_INVALID_DEVICE_PARTITION_COUNT,
        // #endif

        // #ifdef CL_VERSION_2_0
        const CL_INVALID_PIPE_SIZE,
        const CL_INVALID_DEVICE_QUEUE,
        // #endif

        // #ifdef CL_VERSION_2_2
        const CL_INVALID_SPEC_ID,
        const CL_MAX_SIZE_RESTRICTION_EXCEEDED,
        // #endif

        #[doc(hidden)]
        Aborted = 1,
        NvidiaIllegalBufferAction = -9999
    }
}

const fn min_value<const N: usize> (iter: [i32;N]) -> i32 {
    let mut min = i32::MAX;
    let mut i = 0;

    while i < iter.len() {
        if iter[i] < min {
            min = iter[i]
        }

        i += 1;
    }

    min
}

const fn max_value<const N: usize> (iter: [i32;N]) -> i32 {
    let mut max = i32::MIN;
    let mut i = 0;

    while i < N {
        if iter[i] > max {
            max = iter[i]
        }

        i += 1;
    }

    max
}