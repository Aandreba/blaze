use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use opencl_sys::{CL_R, CL_A, CL_LUMINANCE, CL_INTENSITY, CL_RG, CL_RA, CL_RGB, CL_RGBA, CL_ARGB, CL_BGRA, cl_channel_type, CL_UNSIGNED_INT8, CL_UNSIGNED_INT16, CL_UNSIGNED_INT32, CL_SIGNED_INT8, CL_SIGNED_INT16, CL_SIGNED_INT32, CL_FLOAT, CL_SNORM_INT8, CL_SNORM_INT16, CL_UNORM_INT8, CL_UNORM_INT16, cl_image_format, CL_HALF_FLOAT, CL_UNORM_SHORT_565, CL_UNORM_SHORT_555, CL_UNORM_INT_101010, cl_channel_order};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
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

impl ChannelOrder {
    #[inline]
    pub const fn channel_count (&self) -> usize {
        use ChannelOrder::*;

        match self {
            Red | Alpha | Luminance | Intensity => 1,
            RedGreen | RedAlpha => 2,
            RGB => 3,
            RGBA | ARGB | BGRA | ABGR => 4,
            #[cfg(feature = "cl1_1")]
            Rx => 2,
            #[cfg(feature = "cl1_1")]
            RGx => 3,
            #[cfg(feature = "cl2")]
            Depth => 1,
            #[cfg(feature = "cl2")]
            sRGB => 3,
            #[cfg(feature = "cl2")]
            sRGBA | sRGBx | sBGRA | sBGRA | ABGR | RGBx => 4
        }
    }
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

use opencl_sys::{cl_image_desc, cl_mem_object_type};
use crate::{memobj::{MemObjectType, MemObject}};

#[derive(Clone)]
#[non_exhaustive]
pub struct ImageDesc {
    /// Describes the image type.
    pub ty: MemObjectType,
    /// The width of the image in pixels.
    pub width: usize,
    /// The height of the image in pixels.
    pub height: usize,
    /// The depth of the image in pixels. This is only used if the image is a 3D image and must be a value ≥ 1 and ≤ [`Device::image3d_max_depth`].
    pub depth: usize,
    /// The number of images in the image array. This is only used if the image is a 1D or 2D image array. The values for image_array_size, if specified, must be a value ≥ 1 and ≤ [`Device::image_max_array_size`].\
    /// Note that reading and writing 2D image arrays from a kernel with image_array_size = 1 may be lower performance than 2D images.
    pub array_size: usize,
    /// The scan-line pitch in bytes. This must be 0 if host_ptr is NULL and can be either 0 or ≥ image_width * size of element in bytes if host_ptr is not NULL. 
    /// If host_ptr is not NULL and image_row_pitch = 0, image_row_pitch is calculated as image_width * size of element in bytes. If image_row_pitch is not 0, it must be a multiple of the image element size in bytes.
    /// For a 2D image created from a buffer, the pitch specified (or computed if pitch specified is 0) must be a multiple of the maximum of the [`Device::image_pitch_alignment`] value for all devices in the context associated with image_desc->mem_object and that support images.
    pub row_pitch: usize,
    /// The size in bytes of each 2D slice in the 3D image or the size in bytes of each image in a 1D or 2D image array. 
    /// This must be 0 if host_ptr is NULL. If host_ptr is not NULL, image_slice_pitch can be either 0 or ≥ image_row_pitch * image_height for a 2D image array or 3D image and can be either 0 or ≥ image_row_pitch for a 1D image array.
    /// If host_ptr is not NULL and image_slice_pitch = 0, image_slice_pitch is calculated as image_row_pitch * image_height for a 2D image array or 3D image and image_row_pitch for a 1D image array. 
    /// If image_slice_pitch is not 0, it must be a multiple of the image_row_pitch.
    pub slice_pitch: usize,
    /// May refer to a valid buffer or image memory object. mem_object can be a buffer memory object if image_type is CL_MEM_OBJECT_IMAGE1D_BUFFER or CL_MEM_OBJECT_IMAGE2D.
    /// mem_object can be an image object if image_type is CL_MEM_OBJECT_IMAGE2D. Otherwise it must be NULL. The image pixels are taken from the memory objects data store. 
    /// When the contents of the specified memory objects data store are modified, those changes are reflected in the contents of the image object and vice-versa at corresponding synchronization points.
    pub mem_object: Option<MemObject>
}

impl ImageDesc {
    #[inline(always)]
    pub const fn new (ty: MemObjectType, width: usize, height: usize) -> Self {
        Self { 
            ty, width, height,
            depth: 0, 
            array_size: 0, 
            row_pitch: 0, 
            slice_pitch: 0,
            mem_object: None
        }
    }

    #[inline(always)]
    pub const fn to_raw (&self) -> cl_image_desc {
        let buffer = match self.mem_object {
            Some(ref mem_object) => mem_object.id(),
            None => core::ptr::null_mut()
        };

        cl_image_desc {
            image_type: self.ty as cl_mem_object_type,
            image_width: self.width,
            image_height: self.height,
            image_depth: self.depth,
            image_array_size: self.array_size,
            image_row_pitch: self.row_pitch,
            image_slice_pitch: self.slice_pitch,
            num_mip_levels: 0,
            num_samples: 0,
            buffer
        }
    }
}