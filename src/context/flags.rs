use opencl_sys::{cl_context_properties, CL_CONTEXT_PLATFORM};
use rscl_proc::docfg;
use crate::core::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ContextProperties {
    pub platform: Option<Platform>,
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_2")))]
    #[cfg(feature = "cl1_2")]
    pub interop_user_sync: bool
}

#[docfg(not(feature = "cl1_2"))]
impl ContextProperties {
    const SIZE : usize = 1 * 2 + 1;

    #[inline(always)]
    pub fn new (platform: impl Into<Option<Platform>>) -> Self {
        Self {
            platform: platform.into()
        }
    }

    #[inline(always)]
    pub const fn cons_new (platform: Option<Platform>) -> Self {
        Self {
            platform
        }
    }

    #[inline(always)]
    pub fn to_bits (&self) -> Option<[cl_context_properties; Self::SIZE]> {
        if let Some(platform) = self.platform {
            return Some([CL_CONTEXT_PLATFORM, platform.id() as cl_context_properties, 0])
        }

        None
    }
}

#[docfg(feature = "cl1_2")]
impl ContextProperties {
    const SIZE : usize = 2 * 2 + 1;

    #[inline(always)]
    pub fn new (platform: impl Into<Option<Platform>>, interop_user_sync: bool) -> Self {
        Self {
            platform: platform.into(),
            interop_user_sync
        }
    }

    #[inline(always)]
    pub const fn const_new (platform: Option<Platform>, interop_user_sync: bool) -> Self {
        Self {
            platform,
            interop_user_sync
        }
    }

    #[inline(always)]
    pub fn to_bits (&self) -> Option<[cl_context_properties; Self::SIZE]> {
        if let Some(platform) = self.platform {
            return Some(
                [
                    CL_CONTEXT_PLATFORM, platform.id() as cl_context_properties,
                    opencl_sys::CL_CONTEXT_INTEROP_USER_SYNC, self.interop_user_sync as cl_context_properties,
                    0
                ]
            )
        }

        if self.interop_user_sync {
            return Some(
                [
                    CL_CONTEXT_PLATFORM, 0,
                    opencl_sys::CL_CONTEXT_INTEROP_USER_SYNC, self.interop_user_sync as cl_context_properties,
                    0
                ]
            )
        }
        
        None
    }
}

impl Default for ContextProperties {
    #[inline(always)]
    fn default() -> Self {
        Self { 
            platform: None,
            #[cfg(feature = "cl1_2")]
            interop_user_sync: false
        }
    }
}

impl From<Platform> for ContextProperties {
    #[inline(always)]
    fn from(v: Platform) -> Self {
        Self { 
            platform: Some(v),
            #[cfg(feature = "cl1_2")]
            interop_user_sync: false
        }
    }
}

impl From<Option<Platform>> for ContextProperties {
    #[inline(always)]
    fn from(platform: Option<Platform>) -> Self {
        Self { 
            platform,
            #[cfg(feature = "cl1_2")]
            interop_user_sync: false
        }
    }
}