use std::ops::Deref;
use image::{Pixel, Rgb, Rgba, Luma, DynamicImage, ImageBuffer, Primitive};
use num_traits::{NumCast, ToPrimitive};
use super::{ChannelOrder, ChannelType, ImageFormat};

pub trait RawPixel: Pixel {
    const ORDER : ChannelOrder;
    const TY : ChannelType;
    const FORMAT : ImageFormat = ImageFormat::new(Self::ORDER, Self::TY);
}

impl RawPixel for Luma<u8> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::U8;
}

impl RawPixel for Luma<i8> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::I8;
}

impl RawPixel for Luma<u16> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::U16;
}

impl RawPixel for Luma<i16> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::I16;
}

impl RawPixel for Luma<u32> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::U32;
}

impl RawPixel for Luma<i32> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::I32;
}

impl RawPixel for Luma<f32> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::F32;
}

/*
#[docfg(feature = "half")]
impl RawPixel for Luma<::half::f16> {
    const ORDER : ChannelOrder = ChannelOrder::Luminance;
    const TY : ChannelType = ChannelType::F16;
}*/

// RGB
impl RawPixel for Rgb<u8> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::U8;
}

impl RawPixel for Rgb<i8> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::I8;
}

impl RawPixel for Rgb<u16> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::U16;
}

impl RawPixel for Rgb<i16> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::I16;
}

impl RawPixel for Rgb<u32> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::U32;
}

impl RawPixel for Rgb<i32> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::I32;
}

impl RawPixel for Rgb<f32> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::F32;
}

/*#[docfg(feature = "half")]
impl RawPixel for Rgb<::half::f16> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGB;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGB;
    const TY : ChannelType = ChannelType::F16;
}*/

// RGBA
impl RawPixel for Rgba<u8> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::U8;
}

impl RawPixel for Rgba<i8> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::I8;
}

impl RawPixel for Rgba<u16> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::U16;
}

impl RawPixel for Rgba<i16> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::I16;
}

impl RawPixel for Rgba<u32> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::U32;
}

impl RawPixel for Rgba<i32> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::I32;
}

impl RawPixel for Rgba<f32> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::F32;
}

/*#[docfg(feature = "half")]
impl RawPixel for Rgba<::half::f16> {
    #[cfg(feature = "cl2")]
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;
    #[cfg(not(feature = "cl2"))]
    const ORDER : ChannelOrder = ChannelOrder::RGBA;
    const TY : ChannelType = ChannelType::F16;
}*/

/// Convert from one pixel component type to another. For example, convert from `u8` to `f32` pixel values.\
/// Copy and paste from the (unexplicablly) private trait in [`image`] (with added inlining)
pub trait FromPrimitive<Component> {
    /// Converts from any pixel component type to this type.
    fn from_primitive(component: Component) -> Self;
}

impl<T: Primitive> FromPrimitive<T> for T {
    #[inline(always)]
    fn from_primitive(sample: T) -> Self {
        sample
    }
}

// from f32:
// Note that in to-integer-conversion we are performing rounding but NumCast::from is implemented
// as truncate towards zero. We emulate rounding by adding a bias.

impl FromPrimitive<f32> for u8 {
    #[inline]
    fn from_primitive(float: f32) -> Self {
        let inner = (float.clamp(0.0, 1.0) * u8::MAX as f32).round();
        NumCast::from(inner).unwrap()
    }
}

impl FromPrimitive<f32> for u16 {
    #[inline]
    fn from_primitive(float: f32) -> Self {
        let inner = (float.clamp(0.0, 1.0) * u16::MAX as f32).round();
        NumCast::from(inner).unwrap()
    }
}

// from u16:

impl FromPrimitive<u16> for u8 {
    #[inline]
    fn from_primitive(c16: u16) -> Self {
        fn from(c: impl Into<u32>) -> u32 {
            c.into()
        }
        // The input c is the numerator of `c / u16::MAX`.
        // Derive numerator of `num / u8::MAX`, with rounding.
        //
        // This method is based on the inverse (see FromPrimitive<u8> for u16) and was tested
        // exhaustively in Python. It's the same as the reference function:
        //  round(c * (2**8 - 1) / (2**16 - 1))
        NumCast::from((from(c16) + 128) / 257).unwrap()
    }
}

impl FromPrimitive<u16> for f32 {
    #[inline]
    fn from_primitive(int: u16) -> Self {
        (int as f32 / u16::MAX as f32).clamp(0.0, 1.0)
    }
}

// from u8:

impl FromPrimitive<u8> for f32 {
    #[inline]
    fn from_primitive(int: u8) -> Self {
        (int as f32 / u8::MAX as f32).clamp(0.0, 1.0)
    }
}

impl FromPrimitive<u8> for u16 {
    #[inline]
    fn from_primitive(c8: u8) -> Self {
        let x = c8.to_u64().unwrap();
        NumCast::from((x << 8) | x).unwrap()
    }
}

/// Raw pixel that can be convert a [`DynamicImage`] to an [`ImageBuffer`].
pub trait FromDynamic: RawPixel {
    type Container: Deref<Target = [Self::Subpixel]>;

    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container>;
}

impl FromDynamic for Luma<u8> {
    type Container = Vec<u8>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_luma8()
    }
}

impl FromDynamic for Luma<u16> {
    type Container = Vec<u16>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_luma16()
    }
}

impl FromDynamic for Rgb<u8> {
    type Container = Vec<u8>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_rgb8()
    }
}

impl FromDynamic for Rgb<u16> {
    type Container = Vec<u16>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_rgb16()
    }
}

impl FromDynamic for Rgb<f32> {
    type Container = Vec<f32>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_rgb32f()
    }
}

impl FromDynamic for Rgba<u8> {
    type Container = Vec<u8>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_rgba8()
    }
}

impl FromDynamic for Rgba<u16> {
    type Container = Vec<u16>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_rgba16()
    }
}

impl FromDynamic for Rgba<f32> {
    type Container = Vec<f32>;

    #[inline(always)]
    fn from_dynamic (v: DynamicImage) -> ImageBuffer<Self, Self::Container> {
        v.into_rgba32f()
    }
}