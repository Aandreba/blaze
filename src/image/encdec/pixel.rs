use std::{ffi::CStr, ptr::{addr_of}, str::Split};
use ffmpeg_sys_next::{*};

use crate::image::channel::{Rgba, RawPixel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub id: AVPixelFormat,
    pub desc: AVPixFmtDescriptor,
}

impl Pixel {
    #[inline(always)]
    pub const fn new (id: AVPixelFormat, desc: AVPixFmtDescriptor) -> Self {
        Self { id, desc }
    }

    #[inline(always)]
    pub fn from_format (inner: AVPixelFormat) -> Option<Self> {
        let desc = unsafe {
            av_pix_fmt_desc_get(inner)
        };

        if desc.is_null() {
            return None
        }

        unsafe { Some(Self::new(inner, *desc)) }
    }

    #[inline(always)]
    pub fn from_desc (desc: AVPixFmtDescriptor) -> Self {
        let inner = unsafe {
            av_pix_fmt_desc_get_id(addr_of!(desc))
        };

        Self::new(inner, desc)
    }

    #[inline(always)]
    pub fn update_id (&mut self) {
        self.id = unsafe {
            av_pix_fmt_desc_get_id(addr_of!(self.desc))
        };
    }

    #[inline(always)]
    pub fn name (&self) -> Option<&str> {
        if self.desc.name.is_null() {
            return None
        }

        unsafe { CStr::from_ptr(self.desc.name).to_str().ok() }
    }

    #[inline(always)]
    pub fn alias (&self) -> Option<Split<'_, char>> {
        if self.desc.alias.is_null() {
            return None
        }

        unsafe { 
            Some(CStr::from_ptr(self.desc.alias).to_str().ok()?.split(','))
        }
    }

    #[inline(always)]
    pub const fn is_be (&self) -> bool {
        const FLAG : u64 = AV_PIX_FMT_FLAG_BE as u64;
        self.desc.flags & FLAG != 0
    }
    
    #[inline(always)]
    pub const fn is_le (&self) -> bool {
        !self.is_be()
    }

    #[inline(always)]
    pub const fn is_ne (&self) -> bool {
        #[cfg(target_endian = "big")]
        return self.is_be();
        #[cfg(target_endian = "little")]
        return self.is_le();
    }

    #[inline(always)]
    pub fn endianess (&self) -> Endian {
        match self.is_be() {
            true => Endian::Big,
            false => Endian::Little,
        }
    }

    #[inline(always)]
    pub const fn is_bitstream (&self) -> bool {
        const FLAG : u64 = AV_PIX_FMT_FLAG_BITSTREAM as u64;
        self.desc.flags & FLAG != 0
    }

    #[inline(always)]
    pub const fn is_float (&self) -> bool {
        const FLAG : u64 = AV_PIX_FMT_FLAG_FLOAT as u64;
        self.desc.flags & FLAG != 0
    }

    #[inline(always)]
    pub const fn is_rgb (&self) -> bool {
        const FLAG : u64 = AV_PIX_FMT_FLAG_RGB as u64;
        self.desc.flags & FLAG != 0
    }

    #[inline(always)]
    pub const fn has_alpha (&self) -> bool {
        const FLAG : u64 = AV_PIX_FMT_FLAG_ALPHA as u64;
        self.desc.flags & FLAG != 0
    }

    #[inline(always)]
    pub const fn is_hardware_accelerated (&self) -> bool {
        const FLAG : u64 = AV_PIX_FMT_FLAG_HWACCEL as u64;
        self.desc.flags & FLAG != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Endian {
    #[cfg_attr(target_endian = "big", default)]
    Big,
    #[cfg_attr(target_endian = "little", default)]
    Little
}

impl Endian {
    #[cfg(target_endian = "big")]
    pub const NATIVE : Self = Self::Big;
    #[cfg(target_endian = "little")]
    pub const NATIVE : Self = Self::Little;

    #[inline(always)]
    pub const fn is_le (&self) -> bool {
        matches!(self, Self::Little)
    }
    
    #[inline(always)]
    pub const fn is_be (&self) -> bool {
        matches!(self, Self::Big)
    }

    #[inline(always)]
    pub const fn is_native (&self) -> bool {
        #[cfg(target_endian = "big")]
        return self.is_be();
        #[cfg(target_endian = "little")]
        return self.is_le();
    }
}

#[test]
fn test () {
    let mut pix = Rgba::<f32>::FFMPEG.unwrap();
    pix.update_id();
    
    println!("{:#?}", pix)
}