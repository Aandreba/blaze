use core::{mem::MaybeUninit, num::NonZeroUsize};
use opencl_sys::{cl_program, clReleaseProgram, clCreateProgramWithSource, clRetainProgram, clBuildProgram, cl_program_info, clGetProgramInfo, CL_PROGRAM_REFERENCE_COUNT, CL_PROGRAM_CONTEXT, CL_PROGRAM_NUM_DEVICES, CL_PROGRAM_DEVICES, CL_PROGRAM_SOURCE, cl_context};
use crate::context::{Context, Global};
use super::*;

/// OpenCL program
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Program (pub(crate) cl_program);

impl Program {
    #[inline(always)]
    pub fn from_source (source: &str) -> Result<Self> {
        Self::from_source_with(&Global, source)
    }

    #[inline]
    pub fn from_source_with<C: Context> (ctx: &C, source: &str) -> Result<Self> {
        let len = [source.len()].as_ptr();
        let strings = [source.as_ptr().cast()].as_ptr();

        let mut err = 0;
        let id = unsafe {
            clCreateProgramWithSource(ctx.context_id(), 1, strings, len, &mut err)
        };

        if err != 0 {
            return Err(Error::from(err))
        }

        let this = Self(id);
        this.build(ctx)?;

        Ok(this)
    }

    /// Return the program reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_REFERENCE_COUNT)
    }

    /// Return the context specified when the program object is created
    #[inline(always)]
    pub unsafe fn context_id (&self) -> Result<cl_context> {
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
            tri!(clGetProgramInfo(self.0, CL_PROGRAM_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut()))
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
           tri!(clGetProgramInfo(self.0, CL_PROGRAM_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut()))
        }

        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }

    #[inline]
    pub fn binaries (&self) -> Result<Vec<Option<Vec<u8>>>> {
        todo!()
    }

    #[inline(always)]
    fn build<C: Context> (&self, _cx: &C) -> Result<()> {
        let build_result = unsafe {
            clBuildProgram(self.0, 0, core::ptr::null(), core::ptr::null(), None, core::ptr::null_mut())
        };

        if build_result == 0 {
            return Ok(());
        }

        let build_result = Error::from(build_result);

        /*  
        for device in devices {
            let mut len = 0;
            let err = unsafe {
                clGetProgramBuildInfo(self.0, device.0, CL_PROGRAM_BUILD_LOG, 0, core::ptr::null_mut(), &mut len)
            };

            self.parse_error(err, CL_PROGRAM_BUILD_LOG, 0)?;
            if len == 0 { continue }

            let mut result = Vec::<u8>::with_capacity(len);
            let err = unsafe {
                clGetProgramBuildInfo(self.0, device.0, CL_PROGRAM_BUILD_LOG, len, result.as_mut_ptr().cast(), core::ptr::null_mut())
            };

            self.parse_error(err, CL_PROGRAM_BUILD_LOG, len)?;
            unsafe { result.set_len(len) }

            if let Ok(result) = String::from_utf8(result) {
                build_result = build_result.attach_printable(result);
            }
        }*/

        Err(build_result)
    }

    #[inline]
    fn get_info_string (&self, ty: cl_program_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetProgramInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetProgramInfo(self.0, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_program_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetProgramInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }
}

impl Clone for Program {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainProgram(self.0))
        }

        Self(self.0)
    }
}

impl Drop for Program {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseProgram(self.0));
        }
    }
}

unsafe impl Send for Program {}
unsafe impl Sync for Program {}