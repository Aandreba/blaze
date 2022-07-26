use std::ops::*;
use bytemuck::Zeroable;
use num_traits::{NumOps, NumAssignOps, AsPrimitive, Zero, One};
use rscl_proc::{docfg, NumOps, NumOpsAssign};
use crate::{prelude::{RawContext, Result}, buffer::flags::MemAccess, memobj::MemObjectType};
use super::{ChannelType, ChannelOrder, ImageFormat};
use std::{hash::{Hash, Hasher}, mem::MaybeUninit};

/// # Safety
/// - `Self` must have the same size and alignment as `[Channel; CHANNEL_COUNT]`
///     - This might be accomplished with `#[repr(C)]` or `#[repr(transparent)]`
pub unsafe trait RawPixel: Copy + NumOps + NumAssignOps + bytemuck::Zeroable {
    type Channel: RawChannel;

    const ORDER : ChannelOrder;
    const FORMAT : ImageFormat = ImageFormat::new(Self::ORDER, <Self::Channel as RawChannel>::TYPE);
    const CHANNEL_COUNT : usize = Self::ORDER.channel_count();

    fn channels (&self) -> &[Self::Channel];
    fn channels_mut (&mut self) -> &mut [Self::Channel];

    #[inline(always)]
    unsafe fn from_channels_unchecked (v: &[Self::Channel]) -> Self {
        *(v as *const _ as *const Self)
    }

    #[inline(always)]
    fn from_channels (v: &[Self::Channel]) -> Option<Self> {
        if v.len() != Self::CHANNEL_COUNT {
            return None
        }

        unsafe { Some(Self::from_channels_unchecked(v)) }
    }

    #[inline]
    fn is_supported (ctx: &RawContext, access: MemAccess, ty: MemObjectType) -> Result<bool> {
        let iter = ctx.supported_image_formats(access, ty)?;
        Ok(iter.into_iter().any(|x| x == Self::FORMAT))
    }
}

macro_rules! impl_pixel {
    (
        $(
            $(#[docfg(feature = $feat:literal)])?
            #[repr($repr:ident)]
            $name:ident as $v:ident {
                $(
                    $(#[$init:ident])?
                    $field:ident
                ),+
            }
        )+
    ) => {
        $(
            $(#[docfg(feature = $feat)])?
            #[repr($repr)]
            pub struct $name<T> {
                $(
                    pub $field: impl_pixel!(@field $($init)? $field)
                ),+
            }

            $(#[cfg(feature = $feat)])?
            unsafe impl<T: RawChannel> RawPixel for $name<T> {
                type Channel = T;
                const ORDER : ChannelOrder = ChannelOrder::$v;

                #[inline(always)]
                fn channels (&self) -> &[Self::Channel] {
                    unsafe { 
                        core::slice::from_raw_parts (
                            self as *const _ as *const _,
                            Self::CHANNEL_COUNT
                        )
                    }
                }

                #[inline(always)]
                fn channels_mut (&mut self) -> &mut [Self::Channel] {
                    unsafe { 
                        core::slice::from_raw_parts_mut (
                            self as *mut _ as *mut _,
                            Self::CHANNEL_COUNT
                        )
                    }
                }
            }
            
            $(#[cfg(feature = $feat)])?
            impl<T: Clone> Clone for $name<T> {
                #[inline]
                fn clone (&self) -> Self {
                    Self {
                        $(
                            $field: impl_pixel!(@clone $($init)? self $field)
                        ),+
                    }
                }
            }

            $(#[cfg(feature = $feat)])?
            impl<T: PartialEq> PartialEq for $name<T> {
                #[inline]
                fn eq (&self, other: &Self) -> bool {
                    return $(
                        impl_pixel! { @eq $($init)? self other $field }
                    )&+
                }
            }

            $(#[cfg(feature = $feat)])?
            impl<T: Hash> Hash for $name<T> {
                #[inline]
                fn hash<H> (&self, state: &mut H) where H: Hasher {
                    $(
                        impl_pixel! { @hash $($init)? self $field state }
                    )*
                }
            }

            // ARITHMETIC

            impl<T: Add<T, Output = T>> Add for $name<T> {
                type Output = Self;

                #[inline]
                fn add (self, rhs: Self) -> Self::Output {
                    Self {
                        $($field: impl_pixel!(@op $($init)? self rhs $field add)),+
                    }
                }
            }

            impl<T: Sub<T, Output = T>> Sub for $name<T> {
                type Output = Self;

                #[inline]
                fn sub (self, rhs: Self) -> Self::Output {
                    Self {
                        $($field: impl_pixel!(@op $($init)? self rhs $field sub)),+
                    }
                }
            }

            impl<T: Mul<T, Output = T>> Mul for $name<T> {
                type Output = Self;

                #[inline]
                fn mul (self, rhs: Self) -> Self::Output {
                    Self {
                        $($field: impl_pixel!(@op $($init)? self rhs $field mul)),+
                    }
                }
            }

            impl<T: Div<T, Output = T>> Div for $name<T> {
                type Output = Self;

                #[inline]
                fn div (self, rhs: Self) -> Self::Output {
                    Self {
                        $($field: impl_pixel!(@op $($init)? self rhs $field div)),+
                    }
                }
            }

            impl<T: Rem<T, Output = T>> Rem for $name<T> {
                type Output = Self;

                #[inline]
                fn rem (self, rhs: Self) -> Self::Output {
                    Self {
                        $($field: impl_pixel!(@op $($init)? self rhs $field rem)),+
                    }
                }
            }

            // ASSIGN ARITHMETIC

            impl<T: AddAssign<T>> AddAssign for $name<T> {
                #[inline]
                fn add_assign (&mut self, rhs: Self) {
                    $(
                        impl_pixel! { @op_assign $($init)? self rhs $field add_assign }
                    )+
                }
            }

            impl<T: SubAssign<T>> SubAssign for $name<T> {
                #[inline]
                fn sub_assign (&mut self, rhs: Self) {
                    $(
                        impl_pixel! { @op_assign $($init)? self rhs $field sub_assign }
                    )+
                }
            }

            impl<T: MulAssign<T>> MulAssign for $name<T> {
                #[inline]
                fn mul_assign (&mut self, rhs: Self) {
                    $(
                        impl_pixel! { @op_assign $($init)? self rhs $field mul_assign }
                    )+
                }
            }

            impl<T: DivAssign<T>> DivAssign for $name<T> {
                #[inline]
                fn div_assign (&mut self, rhs: Self) {
                    $(
                        impl_pixel! { @op_assign $($init)? self rhs $field div_assign }
                    )+
                }
            }

            impl<T: RemAssign<T>> RemAssign for $name<T> {
                #[inline]
                fn rem_assign (&mut self, rhs: Self) {
                    $(
                        impl_pixel! { @op_assign $($init)? self rhs $field rem_assign }
                    )+
                }
            }

            $(#[cfg(feature = $feat)])?
            impl<T: Copy> Copy for $name<T> {}
            $(#[cfg(feature = $feat)])?
            impl<T: Eq> Eq for $name<T> {}
            $(#[cfg(feature = $feat)])?
            unsafe impl<T: Zeroable> Zeroable for $name<T> {}
        )+
    };

    (@vis) => { pub };
    (@vis uninit) => { };

    (@field uninit $field:ident) => { MaybeUninit<T> };
    (@field $field:ident) => { T };

    (@op uninit $self:ident $rhs:ident $field:ident $op:ident) => { MaybeUninit::uninit() };
    (@op $self:ident $rhs:ident $field:ident $op:ident) => { $self.$field.$op($rhs.$field) };

    (@op_assign uninit $self:ident $rhs:ident $field:ident $op:ident) => {};
    (@op_assign $self:ident $rhs:ident $field:ident $op:ident) => { $self.$field.$op($rhs.$field); };

    (@clone uninit $self:ident $field:ident) => { MaybeUninit::uninit() };
    (@clone $self:ident $field:ident) => { $self.$field.clone() };

    (@eq uninit $self:ident $rhs:ident $field:ident) => { true };
    (@eq $self:ident $rhs:ident $field:ident) => { $self.$field == $rhs.$field };

    (@hash uninit $self:ident $field:ident $state:ident) => {};
    (@hash $self:ident $field:ident $state:ident) => { $self.$field.hash($state); };
}

impl_pixel! {
    #[repr(transparent)]
    Red as Red {
        red
    }

    #[repr(transparent)]
    Alpha as Alpha {
        alpha
    }

    #[docfg(feature = "cl2")]
    #[repr(C)]
    Depth as Depth {
        depth
    }

    #[repr(transparent)]
    Luma as Luminance {
        luma
    }

    #[repr(transparent)]
    Inten as Intensity {
        inten
    }

    #[repr(C)]
    RG as RedGreen {
        red, green
    }

    #[repr(C)]
    RA as RedAlpha {
        red, alpha
    }

    #[repr(C)]
    Rgb as RGB {
        red, green, blue
    }

    #[repr(C)]
    Rgba as RGBA {
        red, green, blue, alpha
    }

    #[repr(C)]
    Argb as ARGB {
        alpha, red, green, blue
    }

    #[repr(C)]
    Bgra as BGRA {
        blue, green, red, alpha
    }

    #[docfg(feature = "cl1_1")]
    #[repr(C)]
    Rx as Rx {
        red,
        #[uninit]
        x
    }

    #[docfg(feature = "cl1_1")]
    #[repr(C)]
    Rgx as RGBx {
        red,
        green,
        #[uninit]
        x
    }

    #[docfg(feature = "cl1_1")]
    #[repr(C)]
    Rgbx as RGBx {
        red,
        green,
        blue,
        #[uninit]
        x
    }

    #[docfg(feature = "cl2")]
    #[repr(C)]
    Abgr as ABGR {
        alpha, blue, green, red
    }

    #[docfg(feature = "cl2")]
    #[repr(C)]
    SRgb as sRGB {
        red, green, blue
    }

    #[docfg(feature = "cl2")]
    #[repr(C)]
    SRgba as sRGBA {
        red, green, blue, alpha
    }

    #[docfg(feature = "cl2")]
    #[repr(C)]
    SBgra as sBGRA {
        blue, green, red, alpha
    }

    #[docfg(feature = "cl2")]
    #[repr(C)]
    SRgbx as sRGBx {
        red,
        green,
        blue,
        #[uninit]
        x
    }
}

/// A type with an associated [`ChannelType`]
pub unsafe trait RawChannel: Copy + Zero + One + Zeroable + NumOps + NumAssignOps + AsPrimitive<f32> {
    const TYPE : ChannelType;
    const MIN : f32;
    const MAX : f32;
    const DELTA : f32 = Self::MAX - Self::MIN;

    #[inline]
    fn cast<U: RawChannel> (self) -> U where f32: AsPrimitive<U> {
        let norm = (self.as_() / Self::DELTA) + Self::MIN;
        f32::as_((norm * U::DELTA) - U::MIN)
    }
}

macro_rules! impl_channel {
    ($($(#[docfg(feature = $feat:literal)])? $ty:ty as $v:ident $(($two:expr))? $(: $min:expr => $max:expr)?),+) => {
        $(
            $(#[rscl_proc::docfg(feature = $feat)])?
            unsafe impl RawChannel for $ty {
                const TYPE : ChannelType = ChannelType::$v;
                const MIN : f32 = impl_channel!(@min $($min)?);
                const MAX : f32 = impl_channel!(@max $($max)?);
            }
        )+
    };

    (@min) => { Self::MIN as f32 };
    (@min $v:expr) => { $v };

    (@max) => { Self::MAX as f32 };
    (@max $v:expr) => { $v };
}

impl_channel! {
    u8 as U8,
    u16 as U16,
    u32 as U32,
    i8 as I8,
    i16 as I16,
    i32 as I32,
    f32 as F32: 0f32 => 1f32,
    Norm<u8> as NormU8: <u8 as RawChannel>::MIN => <u8 as RawChannel>::MAX,
    Norm<u16> as NormU16: <u16 as RawChannel>::MIN => <u16 as RawChannel>::MAX,
    Norm<i8> as NormI8: <i8 as RawChannel>::MIN => <i8 as RawChannel>::MAX,
    Norm<i16> as NormI16: <i16 as RawChannel>::MIN => <i16 as RawChannel>::MAX,
    #[docfg(feature = "half")]
    ::half::f16 as F16: 0f32 => 1f32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, NumOps, NumOpsAssign)]
#[repr(transparent)]
pub struct Norm<T> (pub T);

impl<T: Zero> Zero for Norm<T> {
    #[inline(always)]
    fn zero() -> Self {
        Self(T::zero())
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: One> One for Norm<T> {
    #[inline(always)]
    fn one() -> Self {
        Self(T::one())
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
        Norm(AsPrimitive::<T>::as_(self))
    }
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

unsafe impl<T: Zeroable> Zeroable for Norm<T> {}