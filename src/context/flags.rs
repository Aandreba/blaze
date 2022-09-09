use opencl_sys::{cl_context_properties, CL_CONTEXT_PLATFORM, cl_platform_id};
use crate::core::RawPlatform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ContextProperties {
    /// Specifies the platform to use.
    pub platform: Option<RawPlatform>,
    /// Specifies whether the user is responsible for synchronization between OpenCL and other APIs. Please refer to the specific sections in the OpenCL Extension Specification that describe sharing with other APIs for restrictions on using this flag. If not specified, a default of `false` is assumed.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_2")))]
    #[cfg(feature = "cl1_2")]
    pub interop_user_sync: bool
}

impl ContextProperties {
    cfg_if::cfg_if! {
        if #[cfg(feature = "cl1_2")] {
            const SIZE : usize = 2 * 2 + 1;
        } else {
            const SIZE : usize = 1 * 2 + 1;
        }
    }

    #[inline(always)]
    pub fn new (platform: impl Into<Option<RawPlatform>>, #[cfg(feature = "cl1_2")] interop_user_sync: bool) -> Self {
        Self {
            platform: platform.into(),
            #[cfg(feature = "cl1_2")]
            interop_user_sync
        }
    }

    #[inline(always)]
    pub const fn const_new (platform: Option<RawPlatform>, #[cfg(feature = "cl1_2")] interop_user_sync: bool) -> Self {
        Self {
            platform,
            #[cfg(feature = "cl1_2")]
            interop_user_sync
        }
    }

    #[inline(always)]
    pub fn to_bits (&self) -> Option<[cl_context_properties; Self::SIZE]> {
        if let Some(platform) = self.platform {
            cfg_if::cfg_if! {
                if #[cfg(feature = "cl1_2")] {
                    return Some([
                        CL_CONTEXT_PLATFORM, platform.id() as cl_context_properties,
                        opencl_sys::CL_CONTEXT_INTEROP_USER_SYNC, self.interop_user_sync as cl_context_properties,
                        0
                    ])
                } else {
                    return Some([
                        CL_CONTEXT_PLATFORM, platform.id() as cl_context_properties,
                        0
                    ])
                }
            }
        }

        #[cfg(feature = "cl1_2")]
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

    #[inline]
    pub fn from_bits (bits: &[cl_context_properties]) -> Self {
        if bits.len() == 0 {
            return Self::default()
        }

        match bits[0] {
            CL_CONTEXT_PLATFORM => unsafe {
                let platform = RawPlatform::from_id(bits[1] as cl_platform_id);
                cfg_if::cfg_if! {
                    if #[cfg(feature = "cl1_2")] {
                        match bits[2] {
                            opencl_sys::CL_CONTEXT_INTEROP_USER_SYNC => {
                                let interop_user_sync = bits[3] != 0;
                                Self::new(platform, interop_user_sync)
                            },

                            0 => Self::new(platform, false),
                            _ => panic!()
                        }
                    } else {
                        return Self::new(platform);
                    }
                }
            },

            #[cfg(feature = "cl1_2")]
            opencl_sys::CL_CONTEXT_INTEROP_USER_SYNC => todo!(),

            0 => Self::default(),
            _ => panic!()
        }
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

impl From<RawPlatform> for ContextProperties {
    #[inline(always)]
    fn from(v: RawPlatform) -> Self {
        Self { 
            platform: Some(v),
            #[cfg(feature = "cl1_2")]
            interop_user_sync: false
        }
    }
}

impl From<Option<RawPlatform>> for ContextProperties {
    #[inline(always)]
    fn from(platform: Option<RawPlatform>) -> Self {
        Self { 
            platform,
            #[cfg(feature = "cl1_2")]
            interop_user_sync: false
        }
    }
}