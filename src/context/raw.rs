use std::ptr::addr_of_mut;
use opencl_sys::{cl_context, clCreateContext, clCreateContextFromType, clRetainContext, clReleaseContext};
use crate::core::*;
use super::ContextProperties;

#[repr(transparent)]
pub struct RawContext (cl_context);

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

        Ok(Self(id))
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

        Ok(Self(id))
    }

    #[inline(always)]
    pub const fn from_id (v: cl_context) -> Self {
        Self (v)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_context {
        self.0
    }
}

impl Clone for RawContext {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainContext(self.0))
        }

        Self(self.0.clone())
    }
}

impl Drop for RawContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseContext(self.0))
        }   
    }
}

unsafe impl Send for RawContext {}
unsafe impl Sync for RawContext {}