use std::{ptr::{addr_of_mut, NonNull}, ffi::c_void};
use opencl_sys::{cl_context, clCreateContext, clCreateContextFromType, clRetainContext, clReleaseContext};
use crate::core::{*, device::DeviceType};
use super::ContextProperties;

#[repr(transparent)]
pub struct RawContext (NonNull<c_void>);

impl RawContext {
    pub fn new (props: ContextProperties, devices: &[Device]) -> Result<Self> {
        let num_devices = u32::try_from(devices.len()).unwrap();
        let props = props.to_bits();
        let props = match props {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let mut err = 0;
        let id = unsafe {
            clCreateContext(props, num_devices, devices.as_ptr().cast(), None, core::ptr::null_mut(), addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(Self::from_id(id).unwrap())
    }

    pub fn from_type (props: ContextProperties, ty: DeviceType) -> Result<Self> {
        let props = props.to_bits();
        let props = match props {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let mut err = 0;
        let id = unsafe {
            clCreateContextFromType(props, ty.bits(), None, core::ptr::null_mut(), addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(Self::from_id(id).unwrap())
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (v: cl_context) -> Self {
        Self(NonNull::new_unchecked(v))
    }

    #[inline(always)]
    pub const fn from_id (v: cl_context) -> Option<Self> {
        NonNull::new(v).map(Self)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_context {
        self.0.as_ptr()
    }
}

impl Clone for RawContext {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainContext(self.id()))
        }

        Self(self.0.clone())
    }
}

impl Drop for RawContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseContext(self.id()))
        }   
    }
}

unsafe impl Send for RawContext {}
unsafe impl Sync for RawContext {}