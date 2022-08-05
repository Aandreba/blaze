use std::{ptr::{NonNull, addr_of_mut}, ops::{Deref, DerefMut}};
use ffmpeg_sys_next::*;
use super::{Error, Result, Packet, Frame};

pub struct CodecContext {
    inner: NonNull<AVCodecContext>
}

impl CodecContext {
    #[inline(always)]
    pub fn new (codec: &AVCodec) -> Self {
        Self::try_new(codec).unwrap()
    }

    #[inline]
    pub fn try_new (codec: &AVCodec) -> Option<Self> {
        let inner = unsafe {
            NonNull::new(avcodec_alloc_context3(codec))?
        };

        Some(Self { inner })
    }

    #[inline]
    pub fn try_from_params (params: &AVCodecParameters) -> Result<Self> {
        let codec = unsafe {
            let ptr = NonNull::new(avcodec_find_decoder(params.codec_id))
                .ok_or_else(|| Error::new(AVERROR_INVALIDDATA, "Codec not found"))?;

            ptr.as_ref()
        };

        let mut this = Self::try_new(codec)
            .ok_or_else(|| Error::new(AVERROR_INVALIDDATA, "Codec not found"))?;

        this.copy_params(params)?;
        this.open(codec)?;
        Ok(this)
    }

    #[inline(always)]
    pub fn copy_params (&mut self, params: &AVCodecParameters) -> Result<()> {
        unsafe {
            trii!(avcodec_parameters_to_context(self.inner.as_ptr(), params));
            Ok(())
        }
    }

    #[inline(always)]
    pub fn open (&mut self, codec: &AVCodec) -> Result<()> {
        unsafe {
            trii!(avcodec_open2(self.inner.as_ptr(), codec, core::ptr::null_mut()));
            Ok(())
        }
    }

    #[inline(always)]
    pub fn send_packet (&mut self, packet: &Packet) -> Result<()> {
        unsafe {
            trii!(avcodec_send_packet(self.inner.as_ptr(), packet.as_ptr()));
            Ok(())
        }
    }

    #[inline(always)]
    pub fn receive_frame (&mut self, frame: &mut Frame) -> Result<()> {
        unsafe {
            trii!(avcodec_receive_frame(self.inner.as_ptr(), frame.as_mut_ptr()));
            Ok(())
        }
    }
}

impl Deref for CodecContext {
    type Target = AVCodecContext;
    
    #[inline(always)]
    fn deref (&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl DerefMut for CodecContext {
    #[inline(always)]
    fn deref_mut (&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl Drop for CodecContext {
    #[inline(always)]
    fn drop(&mut self) {
        let mut ptr = self.inner.as_ptr();

        unsafe {
            avcodec_free_context(addr_of_mut!(ptr))
        }
    }
}