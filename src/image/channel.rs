use image::{Pixel, Rgb, Rgba, Luma};
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