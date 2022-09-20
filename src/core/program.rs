use core::{mem::MaybeUninit, num::NonZeroUsize};
use std::{borrow::Cow, ptr::{NonNull, addr_of_mut}, ffi::{c_void}, ops::Deref};
use box_iter::BoxIntoIter;
use opencl_sys::*;
use blaze_proc::docfg;
use crate::{context::{Context, Global}, core::kernel::RawKernel, prelude::RawContext};
use super::*;

/// OpenCL program
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawProgram (NonNull<c_void>);

impl RawProgram {
    #[inline(always)]
    pub fn from_source<'a> (source: impl AsRef<str>, options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[RawKernel]>)> {
        Self::from_source_in(Global::get(), source, options)
    }

    #[inline(always)]
    pub fn from_binary<'a> (source: &[u8], options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[RawKernel]>)> {
        Self::from_binary_in(Global::get(), source, options)
    }

    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn from_il<'a> (source: impl AsRef<[u8]>, options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[RawKernel]>)> {
        Self::from_il_in(Global::get(), source, options)
    }

    pub fn from_source_in<'a, C: Context> (ctx: &C, source: impl AsRef<str>, options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[RawKernel]>)> {
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

        let kernels = this.kernels()?.into_iter().map(|id| unsafe { RawKernel::from_id(id).unwrap() }).collect::<Box<[_]>>();
        Ok((this, kernels))
    }

    pub fn from_binary_in<'a, C: Context> (ctx: &C, source: &[u8], options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[RawKernel]>)> {
        let devices = ctx.as_raw().devices()?;
        let (num_devices, device_list) = (u32::try_from(devices.len()).unwrap(), devices.as_ptr().cast::<cl_device_id>());

        let lengths = vec![source.len(); devices.len()];
        let binaries = vec![source.as_ptr(); devices.len()];

        println!("{:?}", device_list.is_null());
        println!("{:?}\n", num_devices == 0);

        let mut binary_status = vec![CL_SUCCESS; devices.len()];
        let mut err = 0;

        let id = unsafe {
            clCreateProgramWithBinary(ctx.as_raw().id(), num_devices, device_list, lengths.as_ptr(), binaries.as_ptr(), binary_status.as_mut_ptr(), addr_of_mut!(err))
        };

        match ErrorCode::from(err) {
            ErrorCode::Unknown(CL_SUCCESS) => {},
            ErrorCode::Kind(ErrorKind::InvalidValue) => {
                for status in binary_status.into_iter().map(ErrorCode::from) {
                    if status != ErrorCode::Unknown(CL_SUCCESS) {
                        return Err(Error::from(status))
                    }
                }

                return Err(Error::from(ErrorKind::InvalidValue))
            },
            other => return Err(Error::from(other))
        }

        let this = NonNull::new(id).map(Self).unwrap();
        this.build(options.into(), ctx)?;

        let kernels = this.kernels()?.into_iter().map(|id| unsafe { RawKernel::from_id(id).unwrap() }).collect::<Box<[_]>>();
        Ok((this, kernels))
    }

    #[docfg(feature = "cl2_1")]
    pub fn from_il_in<'a, C: Context> (ctx: &C, source: impl AsRef<[u8]>, options: impl Into<Option<&'a str>>) -> Result<(Self, Box<[RawKernel]>)> {
        let source = source.as_ref();

        let mut err = 0;
        let id = unsafe {
            clCreateProgramWithIL(ctx.as_raw().id(), source.as_ptr().cast(), source.len(), &mut err)
        };

        if err != 0 {
            return Err(Error::from(err))
        }

        let this = NonNull::new(id).map(Self).unwrap();
        this.build(options.into(), ctx)?;

        let kernels = this.kernels()?.into_iter().map(|id| unsafe { RawKernel::from_id(id).unwrap() }).collect::<Box<[_]>>();
        Ok((this, kernels))
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_kernel {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const unsafe fn from_id (id: cl_program) -> Option<Self> {
        NonNull::new(id).map(Self)
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_program) -> Self {
        Self(NonNull::new_unchecked(id))
    }
    
    #[inline(always)]
    pub unsafe fn retain (&self) -> Result<()> {
        tri!(clRetainProgram(self.id()));
        Ok(())
    }

    /// Links a set of compiled program objects and libraries for all the devices or a specific device(s) in the OpenCL context and creates an executable.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn link<'a> (input: &[RawProgram], devices: Option<&[RawDevice]>, options: impl Into<Option<&'a str>>) -> Result<Self> {
        Self::link_in(&Global, input, devices, options)
    }

    /// Links a set of compiled program objects and libraries for all the devices or a specific device(s) in the OpenCL context and creates an executable.
    #[docfg(feature = "cl2")]
    pub fn link_in<'a> (ctx: &RawContext, input: &[RawProgram], devices: Option<&[RawDevice]>, options: impl Into<Option<&'a str>>) -> Result<Self> {
        let (num_devices, device_list) = match devices {
            Some(x) => (u32::try_from(x.len()).unwrap(), x.as_ptr().cast()),
            None => (0, core::ptr::null())
        };

        let options = match options.into() {
            Some(x) => {
                let v = std::ffi::CString::new(x).map_err(|e| Error::new(ErrorType::InvalidBuildOptions, e))?;
                Some(v)
            },
            None => None
        };

        let options = match options {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };
        
        let mut err = 0;
        let id = unsafe {
            clLinkProgram (
                ctx.id(), 
                num_devices, device_list, options, 
                u32::try_from(input.len()).unwrap(), input.as_ptr().cast(),
                None, core::ptr::null_mut(), addr_of_mut!(err)
            )
        };

        if err != 0 { return Err(Error::from(err)) }
        Ok(NonNull::new(id).map(Self).unwrap())
    }

    /// Return the program reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_REFERENCE_COUNT)
    }

    /// Return the context specified when the program object is created
    #[inline(always)]
    pub fn context (&self) -> Result<RawContext> {
        let ctx = self.get_info::<cl_context>(CL_PROGRAM_CONTEXT)?;
        unsafe { 
            tri!(clRetainContext(ctx));
            // SAFETY: Context checked to be valid by `clRetainContext`.
            Ok(RawContext::from_id_unchecked(ctx))
        }
    }

    /// Return the number of devices associated with program.
    #[inline(always)]
    pub fn device_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_NUM_DEVICES)
    }

    /// Return the list of devices associated with the program object. This can be the devices associated with context on which the program object has been created or can be a subset of devices that are specified when a progam object is created using clCreateProgramWithBinary.
    #[inline]
    pub fn devices (&self) -> Result<Vec<RawDevice>> {
        let devs = self.get_info_array::<cl_device_id>(CL_PROGRAM_DEVICES)?;
        devs.into_iter().map(|dev| unsafe {
            let dev = RawDevice::from_id(dev).unwrap();
            #[cfg(feature = "cl1_2")]
            dev.retain()?;
            Ok(dev)
        }).try_collect()
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

        let build_result = ErrorCode::from(build_result);

        for device in ctx.queues().into_iter().map(Deref::deref).map(RawCommandQueue::device) {
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
    fn get_info<T: Copy> (&self, ty: cl_program_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetProgramInfo(self.id(), ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }

    #[allow(unused)]
    #[inline]
    fn get_info_array<T: Copy> (&self, ty: cl_program_info) -> Result<Box<[T]>> {
        let mut size = 0;
        unsafe {
            tri!(clGetProgramInfo(self.id(), ty, 0, core::ptr::null_mut(), addr_of_mut!(size)));
        }

        let mut result = Box::new_uninit_slice(size / core::mem::size_of::<T>());
        unsafe {
            tri!(clGetProgramInfo(self.id(), ty, size, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
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
}

impl Clone for RawProgram {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainProgram(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for RawProgram {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseProgram(self.id()));
        }
    }
}

unsafe impl Send for RawProgram {}
unsafe impl Sync for RawProgram {}