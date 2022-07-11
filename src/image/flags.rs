use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use opencl_sys::{CL_R, CL_A, CL_LUMINANCE, CL_INTENSITY, CL_RG, CL_RA, CL_RGB, CL_RGBA, CL_ARGB, CL_BGRA, cl_channel_type, CL_UNSIGNED_INT8, CL_UNSIGNED_INT16, CL_UNSIGNED_INT32, CL_SIGNED_INT8, CL_SIGNED_INT16, CL_SIGNED_INT32, CL_FLOAT, CL_SNORM_INT8, CL_SNORM_INT16, CL_UNORM_INT8, CL_UNORM_INT16, cl_image_format, CL_HALF_FLOAT, CL_UNORM_SHORT_565, CL_UNORM_SHORT_555, CL_UNORM_INT_101010, cl_channel_order};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ImageFormat {
    pub order: ChannelOrder,
    pub ty: ChannelType
}

impl ImageFormat {
    #[inline(always)]
    pub const fn new (order: ChannelOrder, ty: ChannelType) -> Self {
        Self { order, ty }
    }

    #[inline]
    pub fn from_raw (v: cl_image_format) -> Result<Self, FromRawError> {
        let order = ChannelOrder::try_from(v.image_channel_order)?;
        let ty = ChannelType::try_from(v.image_channel_data_type)?;
        Ok(Self { order, ty })
    }

    #[inline(always)]
    pub const fn into_raw (self) -> cl_image_format {
        cl_image_format {
            image_channel_order: self.order as cl_channel_order,
            image_channel_data_type: self.ty as cl_channel_type,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum ChannelType {
    /// Each channel component is a normalized signed 8-bit integer value.
    NormI8 = CL_SNORM_INT8,
    /// Each channel component is a normalized unsigned 8-bit integer value.
    NormU8 = CL_UNORM_INT8,
    /// Each channel component is a normalized signed 16-bit integer value.
    NormI16 = CL_SNORM_INT16,
    /// Each channel component is a normalized unsigned 16-bit integer value.
    NormU16 = CL_UNORM_INT16,
    /// Each channel component is an unnormalized signed 8-bit integer value.
    I8 = CL_SIGNED_INT8,
    /// Each channel component is an unnormalized unsigned 8-bit integer value.
    U8 = CL_UNSIGNED_INT8,
    /// Each channel component is an unnormalized signed 16-bit integer value.
    I16 = CL_SIGNED_INT16,
    /// Each channel component is an unnormalized unsigned 16-bit integer value.
    U16 = CL_UNSIGNED_INT16,
    /// Each channel component is an unnormalized signed 32-bit integer value.
    I32 = CL_SIGNED_INT32,
    /// Each channel component is an unnormalized unsigned 32-bit integer value.
    U32 = CL_UNSIGNED_INT32,
    /// Each channel component is a 16-bit half-float value.
    F16 = CL_HALF_FLOAT,
    /// Each channel component is a single precision floating-point value
    F32 = CL_FLOAT,
    /// Represents a normalized 5-6-5 3-channel RGB image. The channel order must be [`ChannelOrder::RGB`] or [`ChannelOrder::RGBx`].
    U16_565 = CL_UNORM_SHORT_565,
    /// Represents a normalized x-5-5-5 4-channel xRGB image. The channel order must be [`ChannelOrder::RGB`] or [`ChannelOrder::RGBx`].
    U16_555 = CL_UNORM_SHORT_555,
    /// Represents a normalized x-10-10-10 4-channel xRGB image. The channel order must be [`ChannelOrder::RGB`] or [`ChannelOrder::RGBx`].
    U32_10_10_10 = CL_UNORM_INT_101010,
    /// Represents a normalized 10-10-10-2 four-channel RGBA image. The channel order must be [`ChannelOrder::RGBA`].
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2_1")))]
    #[cfg(feature = "cl2_1")]
    U32_10_10_10_2 = opencl_sys::CL_UNORM_INT_101010_2
}

impl ChannelType {
    #[inline(always)]
    pub const fn is_norm (&self) -> bool {
        match self {
            Self::I8 | Self::U8 | Self::I16 | Self::U16 | Self::I32 | Self::U32 | Self::F16 | Self::F32 => false,
            _ => true
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromRawError {
    Order (TryFromPrimitiveError<ChannelOrder>),
    Type (TryFromPrimitiveError<ChannelType>)
}

impl From<TryFromPrimitiveError<ChannelOrder>> for FromRawError {
    #[inline(always)]
    fn from(x: TryFromPrimitiveError<ChannelOrder>) -> Self {
        Self::Order(x)
    }
}

impl From<TryFromPrimitiveError<ChannelType>> for FromRawError {
    #[inline(always)]
    fn from(x: TryFromPrimitiveError<ChannelType>) -> Self {
        Self::Type(x)
    }
}