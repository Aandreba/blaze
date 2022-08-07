use std::{ptr::NonNull, ffi::CString, ops::{Deref, DerefMut, Range}, marker::PhantomData};
use ffmpeg_sys_next::*;
use super::{Result, Packet};

pub struct FormatContext {
    inner: NonNull<AVFormatContext>,
}

impl FormatContext {
    #[inline(always)]
    pub fn new () -> Self {
        Self::try_new().unwrap()
    }

    #[inline(always)]
    pub fn try_new () -> Option<Self> {
        let inner = unsafe {
            NonNull::new(avformat_alloc_context())?
        };

        Some(Self { inner })
    }

    #[inline]
    pub fn open_input (&mut self, path: impl AsRef<str>) -> Result<()> {
        let path = CString::new(path.as_ref()).unwrap();
        
        unsafe {
            trii! {
                avformat_open_input(
                    &mut self.inner.as_ptr(),
                    path.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut()
                )
            };
        }

        Ok(())
    }

    #[inline]
    pub fn find_stream_info (&mut self) -> Result<()> {
        unsafe {
            trii! {
                avformat_find_stream_info(
                    self.inner.as_ptr(),
                    core::ptr::null_mut()
                )
            };
        }

        Ok(())
    }

    #[inline]
    pub fn av_class (&self) -> Option<&AVClass> {
        unsafe {
            let ptr = NonNull::new(self.av_class as *mut _)?;
            Some(ptr.as_ref())
        }
    }

    #[inline]
    pub fn iformat (&self) -> Option<&AVInputFormat> {
        unsafe {
            let ptr = NonNull::new(self.iformat as *mut _)?;
            Some(ptr.as_ref())
        }
    }

    #[inline]
    pub fn iformat_mut (&mut self) -> Option<&mut AVInputFormat> {
        unsafe {
            let mut ptr = NonNull::new(self.iformat as *mut _)?;
            Some(ptr.as_mut())
        }
    }

    #[inline]
    pub fn oformat (&self) -> Option<&AVOutputFormat> {
        unsafe {
            let ptr = NonNull::new(self.oformat as *mut _)?;
            Some(ptr.as_ref())
        }
    }

    #[inline]
    pub fn oformat_mut (&mut self) -> Option<&mut AVOutputFormat> {
        unsafe {
            let mut ptr = NonNull::new(self.oformat as *mut _)?;
            Some(ptr.as_mut())
        }
    }

    #[inline(always)]
    pub fn streams (&self) -> &[&AVStream] {
        unsafe {
            core::slice::from_raw_parts(self.streams as *const _, self.nb_streams as usize)
        }
    }

    #[inline(always)]
    pub fn streams_mut (&mut self) -> &mut [&mut AVStream] {
        unsafe {
            core::slice::from_raw_parts_mut(self.streams as *mut _, self.nb_streams as usize)
        }
    }

    #[inline]
    pub fn nth_stream (&self, n: usize) -> Option<&AVStream> {
        if n >= self.nb_streams as usize {
            return None;
        }

        unsafe {
            debug_assert!(!self.streams.add(n).is_null());
            Some(&**self.streams.add(n))
        }
    }

    #[inline]
    pub fn nth_stream_mut (&mut self, n: usize) -> Option<&mut AVStream> {
        if n >= self.nb_streams as usize {
            return None;
        }

        unsafe {
            debug_assert!(!self.streams.add(n).is_null());
            Some(&mut **self.streams.add(n))
        }
    }

    #[inline]
    pub fn read_frame (&mut self, packet: &mut Packet) -> Result<()> {
        unsafe {
            trii!(av_read_frame(self.inner.as_ptr(), packet.as_mut_ptr()));
            Ok(())
        }
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *const AVFormatContext {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr (&mut self) -> *mut AVFormatContext {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub unsafe fn into_inner (self) -> *mut AVFormatContext {
        self.inner.as_ptr()
    }
}

impl Deref for FormatContext {
    type Target = AVFormatContext;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            self.inner.as_ref()
        }
    }
}

impl DerefMut for FormatContext {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.inner.as_mut()
        }
    }
}

impl Default for FormatContext {
    #[inline(always)]
    fn default () -> Self {
        Self::new()
    }
}

impl Drop for FormatContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            avformat_free_context(self.inner.as_ptr());
        }
    }
}