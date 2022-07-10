use std::{ops::{Deref, DerefMut}};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use opencl_sys::{CL_R, CL_A, CL_LUMINANCE, CL_INTENSITY, CL_RG, CL_RA, CL_RGB, CL_RGBA, CL_ARGB, CL_BGRA, cl_channel_type, CL_UNSIGNED_INT8, CL_UNSIGNED_INT16, CL_UNSIGNED_INT32, CL_SIGNED_INT8, CL_SIGNED_INT16, CL_SIGNED_INT32, CL_FLOAT, CL_SNORM_INT8, CL_SNORM_INT16, CL_UNORM_INT8, CL_UNORM_INT16, cl_image_format};
use rscl_proc::docfg;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ImageFormat {
    pub order: ChannelOrder,
    pub ty: cl_channel_type
}

impl ImageFormat {
    #[inline(always)]
    pub const fn new<T: ChannelType> (order: ChannelOrder) -> Self {
        Self { order, ty: T::FLAG }
    }

    pub fn from_raw (v: cl_image_format) -> Self {
        let order = ChannelOrder::try_from(v.image_channel_order).unwrap();
        Self { order, ty: v.image_channel_data_type }
    }

    #[inline(always)]
    pub fn set_ty<T: ChannelType> (&mut self) {
        self.ty = T::FLAG
    }

    #[inline(always)]
    pub const fn into_raw (self) -> cl_image_format {
        cl_image_format {
            image_channel_order: self.order as u32,
            image_channel_data_type: self.ty,
        }
    }
}

impl Into<cl_image_format> for ImageFormat {
    #[inline(always)]
    fn into(self) -> cl_image_format {
        self.into_raw()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum ChannelOrder {
    /// Single channel image formats where the single channel represents a red component.
    Red = CL_R,
    /// Single channel image formats where the single channel represents a alpha component.
    Alpha = CL_A,
    /// A single channel image format where the single channel represents a depth component.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    Depth = opencl_sys::CL_DEPTH,
    /// A single channel image format where the single channel represents a luminance value.
    /// The luminance value is replicated into the red, green, and blue components.
    Luminance = CL_LUMINANCE,
    /// A single channel image format where the single channel represents an intensity value.
    /// The intensity value is replicated into the red, green, blue, and alpha components.
    Intensity = CL_INTENSITY,
    /// Two channel image formats. 
    /// The first channel represents a red component, and the second channel represents a green component.
    RedGreen = CL_RG,
    /// Two channel image formats. 
    /// The first channel represents a red component, and the second channel represents a alpha component.
    RedAlpha = CL_RA,
    /// A two channel image format, where the first channel represents a red component and the second channel is ignored.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(feature = "cl1_1")]
    Rx = opencl_sys::CL_Rx,
    /// A three channel image format, where the three channels represent red, green, and blue components.
    RGB = CL_RGB,
    /// A three channel image format, where the first two channels represent red and green components and the third channel is ignored.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(feature = "cl1_1")]
    RGx = opencl_sys::CL_RGx,
    /// Four channel image format, where the four channels represent red, green, blue, and alpha components.
    RGBA = CL_RGBA,
    /// Four channel image format, where the four channels represent red, green, blue, and alpha components.
    ARGB = CL_ARGB,
    /// Four channel image format, where the four channels represent red, green, blue, and alpha components.
    BGRA = CL_BGRA,
    /// Four channel image format, where the four channels represent red, green, blue, and alpha components.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    ABGR = opencl_sys::CL_ABGR,
    /// A four channel image format, where the first three channels represent red, green, and blue components and the fourth channel is ignored.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(feature = "cl1_1")]
    RGBx = opencl_sys::CL_RGBx,
    /// A three channel image format, where the three channels represent red, green, and blue components in the sRGB color space.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    #[allow(non_camel_case_types)]
    sRGB = opencl_sys::CL_sRGB,
    /// Four channel image format, where the first three channels represent red, green, and blue components in the sRGB color space. The fourth channel represents an ALPHA component.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    #[allow(non_camel_case_types)]
    sRGBA = opencl_sys::CL_sRGBA,
    /// Four channel image format, where the first three channels represent red, green, and blue components in the sRGB color space. The fourth channel represents an ALPHA component.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    #[allow(non_camel_case_types)]
    sBGRA = opencl_sys::CL_sBGRA,
    /// A four channel image format, where the three channels represent red, green, and blue components in the sRGB color space. The fourth channel is ignored.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    #[allow(non_camel_case_types)]
    sRGBx = opencl_sys::CL_sRGBx,
}

pub trait ChannelType {
    const FLAG : cl_channel_type;
}

impl ChannelType for u8 {
    const FLAG : cl_channel_type = CL_UNSIGNED_INT8;
}

impl ChannelType for u16 {
    const FLAG : cl_channel_type = CL_UNSIGNED_INT16;
}

impl ChannelType for u32 {
    const FLAG : cl_channel_type = CL_UNSIGNED_INT32;
}

impl ChannelType for i8 {
    const FLAG : cl_channel_type = CL_SIGNED_INT8;
}

impl ChannelType for i16 {
    const FLAG : cl_channel_type = CL_SIGNED_INT16;
}

impl ChannelType for i32 {
    const FLAG : cl_channel_type = CL_SIGNED_INT32;
}

impl ChannelType for f32 {
    const FLAG : cl_channel_type = CL_FLOAT;
}

#[docfg(feature = "half")]
impl ChannelType for half::f16 {
    const FLAG : cl_channel_type = opencl_sys::CL_HALF_FLOAT;
}

/// Represents a normalized channel value
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Normalized<T> (pub T);

impl<T> Normalized<T> {
    #[inline(always)]
    pub const fn new (v: T) -> Self {
        Self(v)
    }

    #[inline(always)]
    pub fn into_inner (self) -> T {
        self.0
    }
}

impl<T> Deref for Normalized<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Normalized<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ChannelType for Normalized<i8> {
    const FLAG : cl_channel_type = CL_SNORM_INT8;
}

impl ChannelType for Normalized<i16> {
    const FLAG : cl_channel_type = CL_SNORM_INT16;
}

impl ChannelType for Normalized<u8> {
    const FLAG : cl_channel_type = CL_UNORM_INT8;
}

impl ChannelType for Normalized<u16> {
    const FLAG : cl_channel_type = CL_UNORM_INT16;
}