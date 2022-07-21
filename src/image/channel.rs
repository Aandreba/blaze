use std::{ops::{Deref, DerefMut}, mem::MaybeUninit, hash::Hash};
use image::Primitive;
use num_traits::AsPrimitive;
use rscl_proc::docfg;
use crate::{prelude::RawContext, buffer::flags::MemAccess, memobj::MemObjectType};
use super::{ChannelType, ChannelOrder, ImageFormat};

pub trait RawPixel: Copy {
    type Subpixel: AsChannelType;

    const ORDER : ChannelOrder;
    const FORMAT : ImageFormat = ImageFormat::new(Self::ORDER, <Self::Subpixel as AsChannelType>::TYPE);
    const CHANNEL_COUNT : usize = Self::ORDER.channel_count();

    fn channels (&self) -> &[Self::Subpixel];

    #[inline]
    fn is_supported (ctx: &RawContext, access: MemAccess, ty: MemObjectType) -> bool {
        let iter = match ctx.supported_image_formats(access, ty) {
            Ok(x) => x,
            Err(_) => return false
        };

        iter.into_iter().any(|x| x == Self::FORMAT)
    }
}

/// Single channel pixel formats where the single channel represents a red component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Red<T> {
    pub red: T
}

impl<T: AsChannelType> RawPixel for Red<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::Red;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        core::slice::from_ref(&self.red)
    }
}

/// Single channel pixel formats where the single channel represents an alpha component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Alpha<T> {
    pub alpha: T
}

impl<T: AsChannelType> RawPixel for Alpha<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::Alpha;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        core::slice::from_ref(&self.alpha)
    }
}

/// A single channel pixel format where the single channel represents a depth component.
#[docfg(feature = "cl2")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Depth<T> (pub T);

#[docfg(feature = "cl2")]
impl<T: AsChannelType> RawPixel for Depth<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::Depth;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        core::slice::from_ref(&self.0)
    }
}

/// A single channel pixel format where the single channel represents a luminance value. The luminance value is replicated into the red, green, and blue components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Luma<T> {
    pub luma: T
}

impl<T: AsChannelType> RawPixel for Luma<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::Luminance;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        core::slice::from_ref(&self.luma)
    }
}

/// A single channel pixel format where the single channel represents an intensity value. The intensity value is replicated into the red, green, blue, and alpha components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Inten<T> {
    pub inten: T
}

impl<T: AsChannelType> RawPixel for Inten<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::Intensity;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        core::slice::from_ref(&self.inten)
    }
}

macro_rules! impl_mix {
    ($($field:ident for $name:ident => [$($(#[docfg(feature = $feat:literal)])? $i:ident: ($($var:ident)&+) / $len:literal),+]),+) => {
        $(
            $(
                $(#[docfg(feature = $feat)])?
                impl<T: AsChannelType, U: AsChannelType> From<$i<U>> for $name<T> where f32: AsPrimitive<T> {
                    #[inline]
                    fn from(x: $i<U>) -> Self {
                        let mean = 0f32 $( + x.$var.as_())+ / ($len as f32);
                        let norm = (mean - U::MIN) / U::DELTA;
                        Self {
                            $field: f32::as_((norm * T::DELTA) + T::MIN)
                        }
                    }
                }
            )+
        )+
    };
}

impl_mix! {
    luma for Luma => [
        RG: (red & green) / 2,
        RA: (red) / 1,
        #[docfg(feature = "cl1_1")]
        Rx: (red) / 1,
        RGB: (red & green & blue) / 3,
        #[docfg(feature = "cl1_1")]
        RGx: (red & green) / 2,
        RGBA: (red & green & blue) / 3,
        ARGB: (red & green & blue) / 3,
        BGRA: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        ABGR: (red & green & blue) / 3,
        #[docfg(feature = "cl1_1")]
        RGBx: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        SRGB: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        SRGBA: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        SBGRA: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        SRGBx: (red & green & blue) / 3
    ],

    inten for Inten => [
        RG: (red & green) / 2,
        RA: (red & alpha) / 2,
        #[docfg(feature = "cl1_1")]
        Rx: (red) / 1,
        RGB: (red & green & blue) / 3,
        #[docfg(feature = "cl1_1")]
        RGx: (red & green) / 2,
        RGBA: (red & green & blue & alpha) / 4,
        ARGB: (red & green & blue & alpha) / 4,
        BGRA: (red & green & blue & alpha) / 4,
        #[docfg(feature = "cl2")]
        ABGR: (red & green & blue & alpha) / 4,
        #[docfg(feature = "cl1_1")]
        RGBx: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        SRGB: (red & green & blue) / 3,
        #[docfg(feature = "cl2")]
        SRGBA: (red & green & blue & alpha) / 4,
        #[docfg(feature = "cl2")]
        SBGRA: (red & green & blue & alpha) / 4,
        #[docfg(feature = "cl2")]
        SRGBx: (red & green & blue) / 3
    ]
}

macro_rules! take_mult {
    ($($name:ident => [$($(#[docfg(feature = $feat:literal)])? $i:ident: $($take:ident $(as $ntake:ident)?),+);+]),+) => {
        $(
            $(
                $(#[docfg(feature = $feat)])?
                impl<T: AsChannelType, U: AsChannelType> From<$i<U>> for $name<T> where f32: AsPrimitive<T> {
                    #[inline(always)]
                    fn from(x: $i<U>) -> Self {
                        Self {
                            $(
                                $take: take_mult! { @in x $take $(as $ntake)? }
                            ),+
                        }
                    }
                }
            )+
        )+
    };

    (@in $x:ident $take:ident) => {
        $x.$take.convert()
    };

    (@in $x:ident $take:ident as $ntake:ident) => {
        $x.$ntake.convert()
    };
}

macro_rules! take_multx {
    ($($name:ident => [$($(#[docfg(feature = $feat:literal)])? $i:ident: $($take:ident),+);+]),+) => {
        $(
            $(
                $(#[docfg(feature = $feat)])?
                impl<T: AsChannelType, U: AsChannelType> From<$i<U>> for $name<T> where f32: AsPrimitive<T> {
                    #[inline(always)]
                    fn from(x: $i<U>) -> Self {
                        Self::new($(x.$take.convert()),+)
                    }
                }
            )+
        )+
    }
}

/// The first channel represents a red component, the second channel represents a green component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct RG<T> {
    pub red: T,
    pub green: T
}

impl<T: AsChannelType> RawPixel for RG<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::RedGreen;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 2]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

/// The first channel represents a red component, the second channel represents an alpha component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct RA<T> {
    pub red: T,
    pub alpha: T
}

impl<T: AsChannelType> RawPixel for RA<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::RedAlpha;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 2]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

/// A two channel pixel format, where the first channel represents a red component and the second channel is ignored.
#[docfg(feature = "cl1_1")]
#[derive(Debug, Copy)]
#[repr(C)]
pub struct Rx<T> {
    pub red: T,
    #[doc(hidden)]
    x: MaybeUninit<T>
}

#[cfg(feature = "cl1_1")]
impl<T> Rx<T> {
    #[inline(always)]
    pub const fn new (red: T) -> Self {
        Self { red, x: MaybeUninit::uninit() }
    }
}

#[docfg(feature = "cl1_1")]
impl<T: AsChannelType> RawPixel for Rx<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::Rx;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 2]>());
        core::slice::from_ref(&self.red)
    }
}

#[cfg(feature = "cl1_1")]
impl<T: PartialEq> PartialEq for Rx<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.red == other.red
    }
}

#[cfg(feature = "cl1_1")]
impl<T: PartialOrd> PartialOrd for Rx<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.red.partial_cmp(&other.red)
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Ord> Ord for Rx<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.red.cmp(&other.red)
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Hash> Hash for Rx<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.red.hash(state);
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Clone> Clone for Rx<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { red: self.red.clone(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Default> Default for Rx<T> {
    #[inline(always)]
    fn default() -> Self {
        Self { red: Default::default(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Eq> Eq for Rx<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct RGB<T> {
    pub red: T,
    pub green: T,
    pub blue: T
}

impl<T: AsChannelType> RawPixel for RGB<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::RGB;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 3]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[docfg(feature = "cl1_1")]
#[derive(Debug, Copy)]
#[repr(C)]
pub struct RGx<T> {
    pub red: T,
    pub green: T,
    #[doc(hidden)]
    x: MaybeUninit<T>
}

#[cfg(feature = "cl1_1")]
impl<T> RGx<T> {
    #[inline(always)]
    pub const fn new (red: T, green: T) -> Self {
        Self { red, green, x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: PartialEq> PartialEq for RGx<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.red == other.red && self.green == other.green
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Hash> Hash for RGx<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.red.hash(state);
        self.green.hash(state);
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Clone> Clone for RGx<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { red: self.red.clone(), green: self.green.clone(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Default> Default for RGx<T> {
    #[inline(always)]
    fn default() -> Self {
        Self { red: Default::default(), green: Default::default(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Eq> Eq for RGx<T> {}

#[docfg(feature = "cl1_1")]
impl<T: AsChannelType> RawPixel for RGx<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::RGx;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 3]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, 2) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct RGBA<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    pub alpha: T
}

impl<T: AsChannelType> RawPixel for RGBA<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::RGBA;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct ARGB<T> {
    pub alpha: T,
    pub red: T,
    pub green: T,
    pub blue: T
}

impl<T: AsChannelType> RawPixel for ARGB<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::ARGB;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct BGRA<T> {
    pub blue: T,
    pub green: T,
    pub red: T,
    pub alpha: T
}

impl<T: AsChannelType> RawPixel for BGRA<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::BGRA;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[docfg(feature = "cl2")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct ABGR<T> {
    pub alpha: T,
    pub blue: T,
    pub green: T,
    pub red: T
}

#[docfg(feature = "cl2")]
impl<T: AsChannelType> RawPixel for ABGR<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::ABGR;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[docfg(feature = "cl1_1")]
#[derive(Debug, Copy)]
#[repr(C)]
pub struct RGBx<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    #[doc(hidden)]
    x: MaybeUninit<T>
}

#[cfg(feature = "cl1_1")]
impl<T> RGBx<T> {
    #[inline(always)]
    pub const fn new (red: T, green: T, blue: T) -> Self {
        Self { red, green, blue, x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: PartialEq> PartialEq for RGBx<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.red == other.red && self.green == other.green && self.blue == other.blue
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Hash> Hash for RGBx<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.red.hash(state);
        self.green.hash(state);
        self.blue.hash(state);
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Clone> Clone for RGBx<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { red: self.red.clone(), green: self.green.clone(), blue: self.blue.clone(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Default> Default for RGBx<T> {
    #[inline(always)]
    fn default() -> Self {
        Self { red: Default::default(), green: Default::default(), blue: Default::default(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl1_1")]
impl<T: Eq> Eq for RGBx<T> {}

#[docfg(feature = "cl1_1")]
impl<T: AsChannelType> RawPixel for RGBx<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::RGBx;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, 3) }
    }
}

#[docfg(feature = "cl2")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct SRGB<T> {
    pub red: T,
    pub green: T,
    pub blue: T
}

#[docfg(feature = "cl2")]
impl<T: AsChannelType> RawPixel for SRGB<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::sRGB;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 3]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[docfg(feature = "cl2")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct SRGBA<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    pub alpha: T
}

#[docfg(feature = "cl2")]
impl<T: AsChannelType> RawPixel for SRGBA<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::sRGBA;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[docfg(feature = "cl2")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub struct SBGRA<T> {
    pub blue: T,
    pub green: T,
    pub red: T,
    pub alpha: T
}

#[docfg(feature = "cl2")]
impl<T: AsChannelType> RawPixel for SBGRA<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::sBGRA;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, Self::CHANNEL_COUNT) }
    }
}

#[docfg(feature = "cl2")]
#[derive(Debug, Copy)]
#[repr(C)]
pub struct SRGBx<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    #[doc(hidden)]
    x: MaybeUninit<T>
}

#[cfg(feature = "cl2")]
impl<T> SRGBx<T> {
    #[inline(always)]
    pub const fn new (red: T, green: T, blue: T) -> Self {
        Self { red, green, blue, x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl2")]
impl<T: PartialEq> PartialEq for SRGBx<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.red == other.red && self.green == other.green && self.blue == other.blue
    }
}

#[cfg(feature = "cl2")]
impl<T: Hash> Hash for SRGBx<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.red.hash(state);
        self.green.hash(state);
        self.blue.hash(state);
    }
}

#[cfg(feature = "cl2")]
impl<T: Clone> Clone for SRGBx<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { red: self.red.clone(), green: self.green.clone(), blue: self.blue.clone(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl2")]
impl<T: Default> Default for SRGBx<T> {
    #[inline(always)]
    fn default() -> Self {
        Self { red: Default::default(), green: Default::default(), blue: Default::default(), x: MaybeUninit::uninit() }
    }
}

#[cfg(feature = "cl2")]
impl<T: Eq> Eq for SRGBx<T> {}

#[docfg(feature = "cl2")]
impl<T: AsChannelType> RawPixel for SRGBx<T> {
    type Subpixel = T;
    const ORDER : ChannelOrder = ChannelOrder::sRGBx;

    #[inline(always)]
    fn channels (&self) -> &[Self::Subpixel] {
        assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<[T; 4]>());
        unsafe { core::slice::from_raw_parts(self as *const _ as *const _, 3) }
    }
}

take_mult! {
    Red => [
        Luma: red as luma;
        Inten: red as inten;
        RG: red;
        RA: red;
        #[docfg(feature = "cl1_1")]
        Rx: red;
        RGB: red;
        RGx: red;
        RGBA: red;
        ARGB: red;
        BGRA: red;
        #[docfg(feature = "cl2")]
        ABGR: red;
        #[docfg(feature = "cl1_1")]
        RGBx: red;
        #[docfg(feature = "cl2")]
        SRGB: red;
        #[docfg(feature = "cl2")]
        SRGBA: red;
        #[docfg(feature = "cl2")]
        SBGRA: red;
        #[docfg(feature = "cl2")]
        SRGBx: red
    ],

    Alpha => [
        Inten: alpha as inten;
        RA: alpha;
        RGBA: alpha;
        ARGB: alpha;
        BGRA: alpha;
        #[docfg(feature = "cl2")]
        ABGR: alpha;
        #[docfg(feature = "cl2")]
        SRGBA: alpha;
        #[docfg(feature = "cl2")]
        SBGRA: alpha
    ],

    RA => [
        Inten: red as inten, alpha as inten;
        RGBA: red, alpha;
        ARGB: red, alpha;
        BGRA: red, alpha;
        #[docfg(feature = "cl2")]
        ABGR: red, alpha;
        #[docfg(feature = "cl2")]
        SRGBA: red, alpha;
        #[docfg(feature = "cl2")]
        SBGRA: red, alpha
    ],

    RG => [
        Luma: red as luma, green as luma;
        Inten: red as inten, green as inten;
        RGB: red, green;
        RGx: red, green;
        RGBA: red, green;
        ARGB: red, green;
        BGRA: red, green;
        #[docfg(feature = "cl2")]
        ABGR: red, green;
        #[docfg(feature = "cl1_1")]
        RGBx: red, green;
        #[docfg(feature = "cl2")]
        SRGB: red, green;
        #[docfg(feature = "cl2")]
        SRGBA: red, green;
        #[docfg(feature = "cl2")]
        SBGRA: red, green;
        #[docfg(feature = "cl2")]
        SRGBx: red, green
    ],

    RGB => [
        Luma: red as luma, green as luma, blue as luma;
        Inten: red as inten, green as inten, blue as inten;
        RGBA: red, green, blue;
        ARGB: red, green, blue;
        BGRA: red, green, blue;
        #[docfg(feature = "cl2")]
        ABGR: red, green, blue;
        #[docfg(feature = "cl1_1")]
        RGBx: red, green, blue;
        #[docfg(feature = "cl2")]
        SRGB: red, green, blue;
        #[docfg(feature = "cl2")]
        SRGBA: red, green, blue;
        #[docfg(feature = "cl2")]
        SBGRA: red, green, blue;
        #[docfg(feature = "cl2")]
        SRGBx: red, green, blue
    ],

    RGBA => [
        Inten: red as inten, green as inten, blue as inten, alpha as inten;
        ARGB: red, green, blue, alpha;
        BGRA: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        ABGR: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        SRGBA: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        SBGRA: red, green, blue, alpha
    ],

    ARGB => [
        Inten: red as inten, green as inten, blue as inten, alpha as inten;
        RGBA: red, green, blue, alpha;
        BGRA: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        ABGR: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        SRGBA: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        SBGRA: red, green, blue, alpha
    ],

    BGRA => [
        Inten: red as inten, green as inten, blue as inten, alpha as inten;
        RGBA: red, green, blue, alpha;
        ARGB: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        ABGR: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        SRGBA: red, green, blue, alpha;
        #[docfg(feature = "cl2")]
        SBGRA: red, green, blue, alpha
    ]
}

#[docfg(feature = "cl1_1")]
take_multx! {
    Rx => [
        Luma: luma;    
        Inten: inten;
        Red: red;
        RG: red;
        RA: red;
        RGB: red;
        RGx: red;
        RGBA: red;
        ARGB: red;
        BGRA: red;
        #[docfg(feature = "cl2")]
        ABGR: red;
        RGBx: red;
        #[docfg(feature = "cl2")]
        SRGB: red;
        #[docfg(feature = "cl2")]
        SRGBA: red;
        #[docfg(feature = "cl2")]
        SBGRA: red;
        #[docfg(feature = "cl2")]
        SRGBx: red
    ],

    RGx => [
        Luma: luma, luma;    
        Inten: inten, inten;
        RG: red, green;
        RGB: red, green;
        RGBA: red, green;
        ARGB: red, green;
        BGRA: red, green;
        #[docfg(feature = "cl2")]
        ABGR: red, green;
        RGBx: red, green;
        #[docfg(feature = "cl2")]
        SRGB: red, green;
        #[docfg(feature = "cl2")]
        SRGBA: red, green;
        #[docfg(feature = "cl2")]
        SBGRA: red, green;
        #[docfg(feature = "cl2")]
        SRGBx: red, green
    ],

    RGBx => [
        Luma: luma, luma, luma;
        Inten: inten, inten, inten;
        RGB: red, green, blue;
        RGBA: red, green, blue;
        ARGB: red, green, blue;
        BGRA: red, green, blue;
        #[docfg(feature = "cl2")]
        ABGR: red, green, blue;
        #[docfg(feature = "cl2")]
        SRGB: red, green, blue;
        #[docfg(feature = "cl2")]
        SRGBA: red, green, blue;
        #[docfg(feature = "cl2")]
        SBGRA: red, green, blue;
        #[docfg(feature = "cl2")]
        SRGBx: red, green, blue
    ]
}

#[docfg(feature = "cl2")]
take_mult! {
    ABGR => [
        Inten: red as inten, green as inten, blue as inten, alpha as inten;
        RGBA: red, green, blue, alpha;
        ARGB: red, green, blue, alpha;
        BGRA: red, green, blue, alpha;
        SRGBA: red, green, blue, alpha;
        SBGRA: red, green, blue, alpha
    ],

    SRGB => [
        Luma: red as luma, green as luma, blue as luma;
        Inten: red as inten, green as inten, blue as inten;
        RGB: red, green, blue;
        RGBA: red, green, blue;
        ARGB: red, green, blue;
        BGRA: red, green, blue;
        ABGR: red, green, blue;
        RGBx: red, green, blue;
        SRGBA: red, green, blue;
        SBGRA: red, green, blue;
        SRGBx: red, green, blue
    ],

    SRGBA => [
        Inten: red as inten, green as inten, blue as inten, alpha as inten;
        RGBA: red, green, blue, alpha;
        ARGB: red, green, blue, alpha;
        BGRA: red, green, blue, alpha;
        ABGR: red, green, blue, alpha;
        SBGRA: red, green, blue, alpha
    ],

    SBGRA => [
        Inten: red as inten, green as inten, blue as inten, alpha as inten;
        RGBA: red, green, blue, alpha;
        ARGB: red, green, blue, alpha;
        BGRA: red, green, blue, alpha;
        ABGR: red, green, blue, alpha;
        SRGBA: red, green, blue, alpha
    ]
}

#[docfg(feature = "cl2")]
take_multx! {
    SRGBx => [
        Luma: luma, luma, luma;
        Inten: inten, inten, inten;
        RGBx: red, green, blue;
        RGB: red, green, blue;
        RGBA: red, green, blue;
        ARGB: red, green, blue;
        BGRA: red, green, blue;
        ABGR: red, green, blue;
        SRGB: red, green, blue;
        SRGBA: red, green, blue;
        SBGRA: red, green, blue
    ]
}

// Conversions from and into `image` crate
impl<T: Primitive + AsChannelType> From<image::Luma<T>> for Luma<T> {
    #[inline(always)]
    fn from(x: image::Luma<T>) -> Self {
        Luma { luma: x.0[0] }
    }
}

impl<T: Primitive + AsChannelType> Into<image::Luma<T>> for Luma<T> {
    #[inline(always)]
    fn into(self) -> image::Luma<T> {
        image::Luma([self.luma])
    }
}

/// A type that can be represented as an image's channel type
pub trait AsChannelType: AsPrimitive<f32> {
    const TYPE : ChannelType;
    const MIN : f32;
    const MAX : f32;

    const DELTA : f32 = Self::MAX - Self::MIN;

    #[inline]
    fn convert<U: AsChannelType> (self) -> U where f32: AsPrimitive<U> {
        // Optimize self converssion
        /*if TypeId::of::<Self>() == TypeId::of::<U>() {
            return unsafe { *(addr_of!(self) as *const U) }
        }*/

        let norm = (self.as_() - Self::MIN) / Self::DELTA;
        f32::as_((norm * U::DELTA) + U::MIN)
    }
}

impl AsChannelType for u8 {
    const TYPE : ChannelType = ChannelType::U8;
    const MIN : f32 = Self::MIN as f32;
    const MAX : f32 = Self::MAX as f32;
}

impl AsChannelType for i8 {
    const TYPE : ChannelType = ChannelType::I8;
    const MIN : f32 = Self::MIN as f32;
    const MAX : f32 = Self::MAX as f32;
}

impl AsChannelType for u16 {
    const TYPE : ChannelType = ChannelType::U16;
    const MIN : f32 = Self::MIN as f32;
    const MAX : f32 = Self::MAX as f32;
}

impl AsChannelType for i16 {
    const TYPE : ChannelType = ChannelType::I16;
    const MIN : f32 = Self::MIN as f32;
    const MAX : f32 = Self::MAX as f32;
}

impl AsChannelType for u32 {
    const TYPE : ChannelType = ChannelType::U32;
    const MIN : f32 = Self::MIN as f32;
    const MAX : f32 = Self::MAX as f32;
}

impl AsChannelType for i32 {
    const TYPE : ChannelType = ChannelType::I32;
    const MIN : f32 = Self::MIN as f32;
    const MAX : f32 = Self::MAX as f32;
}

#[docfg(feature = "half")]
impl AsChannelType for ::half::f16 {
    const TYPE : ChannelType = ChannelType::F16;
    const MIN : f32 = 0f32;
    const MAX : f32 = 1f32;
}

impl AsChannelType for f32 {
    const TYPE : ChannelType = ChannelType::F32;
    const MIN : f32 = 0f32;
    const MAX : f32 = 1f32;
}

/// Normalized channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Norm<T> (pub T);

impl AsChannelType for Norm<u8> {
    const TYPE : ChannelType = ChannelType::NormU8;
    const MIN : f32 = u8::MIN as f32;
    const MAX : f32 = u8::MAX as f32;
}

impl AsChannelType for Norm<i8> {
    const TYPE : ChannelType = ChannelType::NormI8;
    const MIN : f32 = i8::MIN as f32;
    const MAX : f32 = i8::MAX as f32;
}

impl AsChannelType for Norm<u16> {
    const TYPE : ChannelType = ChannelType::NormU16;
    const MIN : f32 = u16::MIN as f32;
    const MAX : f32 = u16::MAX as f32;
}

impl AsChannelType for Norm<i16> {
    const TYPE : ChannelType = ChannelType::NormI16;
    const MIN : f32 = i16::MIN as f32;
    const MAX : f32 = i16::MAX as f32;
}

impl<T> Deref for Norm<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Norm<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsPrimitive<f32>> AsPrimitive<f32> for Norm<T> {
    #[inline(always)]
    fn as_(self) -> f32 {
        self.0.as_()
    }
}

impl<T: 'static + Copy> AsPrimitive<Norm<T>> for f32 where f32: AsPrimitive<T> {
    #[inline(always)]
    fn as_(self) -> Norm<T> {
        Norm(self.as_())
    }
}

#[test]
fn casting () {
    let test = RGx::new(0.9f32, 0.5);
    println!("{:?}", test.channels())
}