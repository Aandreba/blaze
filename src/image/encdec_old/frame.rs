use std::{ptr::{NonNull, addr_of_mut}, ops::{Deref, DerefMut, Index}};
use ffmpeg_sys_next::*;
use super::Result;

pub struct Packet {
    inner: NonNull<AVPacket>
}

impl Packet {
    #[inline(always)]
    pub fn new () -> Self {
        Self::try_new().unwrap()
    }

    #[inline(always)]
    pub fn try_new () -> Option<Self> {
        let inner = unsafe {
            NonNull::new(av_packet_alloc())?
        };

        Some(Self { inner })
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *const AVPacket {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr (&mut self) -> *mut AVPacket {
        self.inner.as_ptr()
    }
}

pub struct Frame {
    inner: NonNull<AVFrame>
}

impl Frame {
    #[inline(always)]
    pub fn new () -> Self {
        Self::try_new().unwrap()
    }
    
    #[inline(always)]
    pub fn try_new () -> Option<Self> {
        let inner = unsafe {
            NonNull::new(av_frame_alloc())?
        };

        Some(Self { inner })
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *const AVFrame {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr (&mut self) -> *mut AVFrame {
        self.inner.as_ptr()
    }
}

impl Deref for Packet {
    type Target = AVPacket;
    
    #[inline(always)]
    fn deref (&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl DerefMut for Packet {
    #[inline(always)]
    fn deref_mut (&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl Deref for Frame {
    type Target = AVFrame;
    
    #[inline(always)]
    fn deref (&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl DerefMut for Frame {
    #[inline(always)]
    fn deref_mut (&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl Default for Packet {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Frame {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Packet {
    #[inline(always)]
    fn drop(&mut self) {
        let mut ptr = self.inner.as_ptr();
        unsafe {
            av_packet_free(addr_of_mut!(ptr));
        }
    }
}

impl Drop for Frame {
    #[inline(always)]
    fn drop(&mut self) {
        let mut ptr = self.inner.as_ptr();
        unsafe {
            av_frame_free(addr_of_mut!(ptr));
        }
    }
}