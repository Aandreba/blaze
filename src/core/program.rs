use core::{mem::MaybeUninit, num::NonZeroUsize};
use std::{borrow::Cow, ptr::NonNull, ffi::c_void};
use opencl_sys::*;
use crate::{context::{Context, Global}, core::kernel::Kernel};
use super::*;

/// OpenCL program
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Program (NonNull<c_void>);

impl Program {
    #[inline(always)]
    pub fn from_source<'a> (source: impl AsRef<str>, options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[Kernel]>)> {
        Self::from_source_in(&Global, source, options)
    }

    #[inline]
    pub fn from_source_in<'a, C: Context> (ctx: &C, source: impl AsRef<str>, options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[Kernel]>)> {
        let source = source.as_ref();
        let len = [source.len()].as_ptr();
        let strings = [source.as_ptr().cast()].as_ptr();

        let mut err = 0;
        let id = unsafe {
            clCreateProgramWithSource(ctx.as_raw().id(), 1, strings, len, &mut err)
        };

        if err != 0 {
            return Err(Error::from(err))
        }

        let this = NonNull::new(id).map(Self).unwrap();
        this.build(options.into(), ctx)?;

        let kernels = this.kernels()?.into_iter().map(|id| Kernel::from_id(*id).unwrap()).collect::<Box<[_]>>();
        Ok((this, kernels))
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_kernel {
        self.0.as_ptr()
    }

    #[inline]
    fn kernels (&self) -> Result<Box<[cl_kernel]>> {
        let mut len = 0;
        unsafe {
            tri!(clCreateKernelsInProgram(self.id(), 0, core::ptr::null_mut(), &mut len));
            let mut kernels = Box::new_uninit_slice(len as usize);
            tri!(clCreateKernelsInProgram(self.id(), len, kernels.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(kernels.assume_init())
        }
    }

    /// Return the program reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_REFERENCE_COUNT)
    }

    /// Return the context specified when the program object is created
    #[inline(always)]
    pub fn context_id (&self) -> Result<cl_context> {
        let ctx : cl_context = self.get_info(CL_PROGRAM_CONTEXT)?;
        Ok(ctx)
    }

    /// Return the number of devices associated with program.
    #[inline(always)]
    pub fn device_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_NUM_DEVICES)
    }

    /// Return the list of devices associated with the program object. This can be the devices associated with context on which the program object has been created or can be a subset of devices that are specified when a progam object is created using clCreateProgramWithBinary.
    #[inline]
    pub fn devices (&self) -> Result<Vec<Device>> {
        let count = self.device_count()?;
        let mut result = Vec::<Device>::with_capacity(count as usize);
        let size = result.capacity().checked_mul(core::mem::size_of::<Device>()).expect("Too many devices");

        unsafe {
            tri!(clGetProgramInfo(self.id(), CL_PROGRAM_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut()))
        }
        
        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }

    /// Return the program source code
    #[inline(always)]
    pub fn source (&self) -> Result<String> {
        self.get_info_string(CL_PROGRAM_SOURCE)
    }

    /// Returns an array that contains the size in bytes of the program binary for each device associated with program. The size of the array is the number of devices associated with program. If a binary is not available for a device(s), a size of zero is returned.
    #[inline]
    pub fn binary_sizes (&self) -> Result<Vec<Option<NonZeroUsize>>> {
        let count = self.device_count()?;
        let mut result = Vec::<Option<NonZeroUsize>>::with_capacity(count as usize);
        let size = result.capacity().checked_mul(core::mem::size_of::<usize>()).expect("Too many binaries");

        unsafe {
           tri!(clGetProgramInfo(self.id(), CL_PROGRAM_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut()))
        }

        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }

    #[inline]
    pub fn binaries (&self) -> Result<Vec<Option<Vec<u8>>>> {
        todo!()
    }

    fn build<C: Context> (&self, options: Option<&str>, ctx: &C) -> Result<()> {
        let options : Option<Cow<'static, str>> = match options {
            Some(x) => {
                let mut x = x.to_string();
                #[cfg(feature = "cl1_2")]
                x.push_str(" -cl-kernel-arg-info");
                x.push('\0');
                Some(Cow::Owned(x))
            },

            #[cfg(feature = "cl1_2")]
            None => Some(Cow::Borrowed("-cl-kernel-arg-info\0")),
            #[cfg(not(feature = "cl1_2"))]
            None => None
        };

        let ops = match options {
            Some(x) => x.as_ptr(),
            #[cfg(all(debug_assertions, feature = "cl1_2"))]
            None => unreachable!(),
            #[cfg(all(not(debug_assertions), feature = "cl1_2"))]
            None => unsafe { unreachable_unchecked() },
            #[cfg(not(feature = "cl1_2"))]
            None => core::ptr::null()
        };

        let build_result = unsafe {
            clBuildProgram(self.id(), 0, core::ptr::null(), ops.cast(), None, core::ptr::null_mut())
        };

        if build_result == 0 {
            return Ok(());
        }

        let build_result = ErrorType::from(build_result);

        for device in ctx.queues().into_iter().map(RawCommandQueue::device) {
            let device = device?;
            
            let mut len = 0;
            unsafe {
                tri!(clGetProgramBuildInfo(self.id(), device.id(), CL_PROGRAM_BUILD_LOG, 0, core::ptr::null_mut(), &mut len))
            };

            if len == 0 { continue }

            let mut result = Box::<[u8]>::new_uninit_slice(len);
            unsafe {
                tri!(clGetProgramBuildInfo(self.id(), device.id(), CL_PROGRAM_BUILD_LOG, len, result.as_mut_ptr().cast(), core::ptr::null_mut()))
            };

            let result = unsafe { result.assume_init() };
            return Err(Error::new(build_result, String::from_utf8_lossy(&result)));
        }

        Err(build_result.into())
    }

    #[inline]
    fn get_info_string (&self, ty: cl_program_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetProgramInfo(self.id(), ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetProgramInfo(self.id(), ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_program_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetProgramInfo(self.id(), ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }
}

impl Clone for Program {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainProgram(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for Program {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseProgram(self.id()));
        }
    }
}

unsafe impl Send for Program {}
unsafe impl Sync for Program {}