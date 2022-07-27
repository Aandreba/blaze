use std::{path::Path, ptr::{addr_of_mut, NonNull}};
use ffmpeg_sys_next::*;
use ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_VIDEO;
use crate::image::channel::Rgb;

use super::{error::{Result, Error, ErrorKind}, path_str, alloc::{new_frame, frame_to_rect}};

pub struct FfmpegInputFile {
    format_ctx: NonNull<AVFormatContext>,
    codec_ctx: NonNull<AVCodecContext>
}

impl FfmpegInputFile {
    pub fn input (path: impl AsRef<Path>) -> Result<Self> {
        let path = path_str(path).map_err(|e| Error::new(super::error::ErrorKind::InvalidData, e))?;
        let mut format_ctx = core::ptr::null_mut();

        unsafe {
            ffmpeg_tri! {
                avformat_open_input(addr_of_mut!(format_ctx), path.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut());
                avformat_find_stream_info(format_ctx, core::ptr::null_mut())
            }

            av_dump_format(format_ctx, 0, path.as_ptr(), 0);
        }
        
        let mut format_ctx = NonNull::new(format_ctx).unwrap();
        let mut stream_idx = None;

        for i in unsafe { 0..format_ctx.as_mut().nb_streams } {
            unsafe {
                let stream = &**format_ctx.as_mut().streams.add(i as usize);
                if stream.codec.is_null() {
                    continue;
                }

                let codec = &*stream.codec;
                if codec.codec_type == AVMEDIA_TYPE_VIDEO {
                    stream_idx = Some(i);
                    break
                }
            }
        }

        if stream_idx.is_none() {
            return Err(Error::from_kind(ErrorKind::StreamNotFound));
        }

        let stream_idx = unsafe { stream_idx.unwrap_unchecked() };
        let codec_ctx = unsafe { &**format_ctx.as_mut().streams.add(stream_idx as usize) };
        let codec_ctx = NonNull::new(codec_ctx.codec).unwrap();

        Ok(Self { format_ctx, codec_ctx })
    }

    pub fn decoder (&self) -> Result<NonNull<AVCodec>> {
        unsafe {
            let codec = NonNull::new(avcodec_find_decoder(self.codec_ctx.as_ref().codec_id)).ok_or_else(|| Error::new(ErrorKind::Unknown, "Unknown codec"))?;
            ffmpeg_tri!(avcodec_open2(self.codec_ctx.as_ptr(), codec.as_ptr(), core::ptr::null_mut()));
            Ok(codec)
        }
    }
}

impl Drop for FfmpegInputFile {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            ffmpeg_tri_panic! {
                avcodec_close(self.codec_ctx.as_ptr())
            }

            avformat_close_input(addr_of_mut!(self.format_ctx).cast())
        }
    }
}

#[test]
fn test () {
    let file = FfmpegInputFile::input("tests/test2.jpg").unwrap();
    let dec = file.decoder().unwrap();

    let mut frame = new_frame();
    frame.width = 120;
    frame.height = 120;
    
    let rect = frame_to_rect::<Rgb<u8>>(&frame).unwrap();    
    todo!()
}