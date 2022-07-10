flat_mod!(flags);

pub mod desc;
pub use desc::ImageDesc;

use rscl_proc::docfg;
use std::{ptr::{NonNull, addr_of_mut, addr_of}, ffi::c_void};
use crate::{core::*, context::RawContext, buffer::flags::FullMemFlags};

#[derive(Clone)]
#[repr(transparent)]
pub struct RawImage (MemObject);

impl RawImage {
    #[docfg(feature = "cl1_2")]
    pub unsafe fn new (ctx: &RawContext, flags: FullMemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let image_format = format.into_raw();
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        let id = opencl_sys::clCreateImage(ctx.id(), flags, addr_of!(image_format), image_desc, host_ptr, addr_of_mut!(err));
    }
}