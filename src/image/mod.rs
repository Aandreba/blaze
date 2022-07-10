flat_mod!(flags, complex);
pub mod channel;
//pub mod events;

pub mod desc;
pub use desc::ImageDesc;

use rscl_proc::docfg;
use std::{ptr::{NonNull, addr_of_mut}, ffi::c_void, ops::Deref};
use crate::{core::*, context::RawContext, buffer::flags::FullMemFlags};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RawImage (MemObject);

impl RawImage {
    #[docfg(feature = "cl1_2")]
    pub unsafe fn new (ctx: &RawContext, flags: FullMemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        use std::ptr::addr_of;
        
        let image_format = format.into_raw();
        let image_desc = desc.to_raw();
        
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        let id = opencl_sys::clCreateImage(ctx.id(), flags, addr_of!(image_format), addr_of!(image_desc), host_ptr, addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        let id = MemObject::from_id(id).unwrap();
        Ok(Self(id))
    }

    #[cfg_attr(feature = "cl1_2", deprecated(note = "use `new`"))]
    pub unsafe fn new_2d (ctx: &RawContext, flags: FullMemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let mut image_format = format.into_raw();
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        #[allow(deprecated)]
        let id = opencl_sys::clCreateImage2D(ctx.id(), flags, addr_of_mut!(image_format), desc.width, desc.height, desc.row_pitch, host_ptr, addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        let id = MemObject::from_id(id).unwrap();
        Ok(Self(id))
    }

    #[cfg_attr(feature = "cl1_2", deprecated(note = "use `new`"))]
    pub unsafe fn new_3d (ctx: &RawContext, flags: FullMemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let mut image_format = format.into_raw();
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        #[allow(deprecated)]
        let id = opencl_sys::clCreateImage3D(ctx.id(), flags, addr_of_mut!(image_format), desc.width, desc.height, desc.depth, desc.row_pitch, desc.slice_pitch, host_ptr, addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        let id = MemObject::from_id(id).unwrap();
        Ok(Self(id))
    }
}

impl Deref for RawImage {
    type Target = MemObject;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}