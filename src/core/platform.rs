use crate::non_null_const;

use super::*;
use blaze_proc::docfg;
use core::mem::MaybeUninit;
use opencl_sys::*;
use std::{ffi::c_void, ptr::NonNull};

lazy_static! {
    static ref PLATFORMS: Vec<RawPlatform> = unsafe {
        let mut cnt = 0;
        tri_panic!(clGetPlatformIDs(0, core::ptr::null_mut(), &mut cnt));
        let cnt_size = usize::try_from(cnt).unwrap();

        let mut result = Vec::<RawPlatform>::with_capacity(cnt_size);
        tri_panic!(clGetPlatformIDs(
            cnt,
            result.as_mut_ptr().cast(),
            core::ptr::null_mut()
        ));
        result.set_len(cnt_size);

        result
    };
}

/// OpenCL platform
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawPlatform(NonNull<c_void>);

impl RawPlatform {
    #[inline(always)]
    pub const fn id(&self) -> cl_platform_id {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked(id: cl_platform_id) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    #[inline(always)]
    pub const unsafe fn from_id(id: cl_platform_id) -> Option<Self> {
        match non_null_const(id) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    /// Returns the profile name supported by the implementation.
    #[inline(always)]
    pub fn profile(&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_PROFILE)
    }

    /// Returns the OpenCL version supported by the implementation.
    #[inline(always)]
    pub fn version(&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_VERSION)
    }

    /// Platform name string.
    #[inline(always)]
    pub fn name(&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_NAME)
    }

    /// Platform vendor string.
    #[inline(always)]
    pub fn vendor(&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_VENDOR)
    }

    /// Returns a list of extension names (the extension names themselves do not contain any spaces) supported by the platform. Extensions defined here must be supported by all devices associated with this platform.
    #[inline(always)]
    pub fn extensions(&self) -> Result<Vec<String>> {
        Ok(self
            .get_info_string(CL_PLATFORM_EXTENSIONS)?
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>())
    }

    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn host_timer_resolution(&self) -> Result<u64> {
        self.get_info_bits(opencl_sys::CL_PLATFORM_HOST_TIMER_RESOLUTION)
    }

    #[inline(always)]
    pub fn all() -> &'static [RawPlatform] {
        &PLATFORMS
    }

    #[inline]
    fn get_info_string(&self, ty: cl_platform_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetPlatformInfo(
                self.id(),
                ty,
                0,
                core::ptr::null_mut(),
                &mut len
            ));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetPlatformInfo(
                self.id(),
                ty,
                len * core::mem::size_of::<cl_uchar>(),
                result.as_mut_ptr().cast(),
                core::ptr::null_mut()
            ));

            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn get_info_bits<T: Copy>(&self, ty: cl_platform_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();

        unsafe {
            tri!(clGetPlatformInfo(
                self.id(),
                ty,
                core::mem::size_of::<T>(),
                value.as_mut_ptr().cast(),
                core::ptr::null_mut()
            ));
            Ok(value.assume_init())
        }
    }
}

#[docfg(feature = "cl1_2")]
impl Drop for RawPlatform {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clUnloadPlatformCompiler(self.id()));
        }
    }
}

unsafe impl Send for RawPlatform {}
unsafe impl Sync for RawPlatform {}
