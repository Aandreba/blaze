use std::ops::Deref;
use image::{Pixel, Rgb, Rgba, Luma, DynamicImage, ImageBuffer};
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