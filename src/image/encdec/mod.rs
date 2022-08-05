// https://github.com/leandromoreira/ffmpeg-libav-tutorial#learn-ffmpeg-libav-the-hard-way
#![allow(unused)]

use std::{ffi::CStr, ptr::NonNull, path::Path, fs::File, io::Write};
use ffmpeg_sys_next::{avcodec_find_decoder, AVERROR_INVALIDDATA, AVPixelFormat, sws_scale};
use crate::prelude::Rect2D;

use super::channel::FfmpegPixel;

macro_rules! trii {
    ($e:expr) => {{
        super::Error::try_from_id($e)?;
    }};
}

flat_mod!(error, format, codec, frame, swr);

pub fn decode_image<P: FfmpegPixel> (path: impl AsRef<str>) -> Result<()> {
    let mut format_ctx = FormatContext::new();
    format_ctx.open_input(path)?;
    format_ctx.find_stream_info()?;
   
    let out_format = format_ctx.iformat().unwrap();
    unsafe {
        println!("{:?}", CStr::from_ptr(out_format.name));
    }
    
    let stream = format_ctx.nth_stream_mut(0).unwrap();
    let params = unsafe {
        if stream.codecpar.is_null() {
            return Err(Error::new(AVERROR_INVALIDDATA, "Codec parameters not found"));
        }

        &mut *stream.codecpar
    };

    let mut codec_ctx = CodecContext::try_from_params(params)?;
    let mut packet = Packet::new();
    let mut frame = Frame::new();

    let sws_ctx = get_sws_context(
        codec_ctx.pix_fmt, P::PIX_FMT, 
        codec_ctx.width, codec_ctx.height
    ).unwrap();

    if format_ctx.read_frame(&mut packet).is_ok() {
        codec_ctx.send_packet(&packet)?;
        codec_ctx.receive_frame(&mut frame)?;
        
        /*unsafe {
            trii!(sws_scale(sws_ctx.as_ptr(), frame.data.as_ptr().cast(), frame.linesize.as_ptr(), srcSliceY, srcSliceH, dst, dstStride));
        }*/
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::ptr::addr_of;
    use ffmpeg_sys_next::{av_pix_fmt_desc_get_id, AVPixelFormat};
    use crate::{image::ImageFormat, prelude::*};
    use super::decode_image;

    #[test]
    fn test () {
        //decode_image("tests/index.jpeg").unwrap();
    }

    #[test]
    fn get_format () {
        let ctx = SimpleContext::default().unwrap();
        let formats = ctx.supported_image_formats(MemAccess::default(), crate::memobj::MemObjectType::Image2D).unwrap();

        for format in formats {
            if let Some(desc) = format.ffmpeg_pixel_desc() {
                println!("{format:?}: {desc:?}\n");

                let ffmpeg_format = unsafe {
                    av_pix_fmt_desc_get_id(addr_of!(desc))
                };

                if ffmpeg_format != AVPixelFormat::AV_PIX_FMT_NONE {
                    println!("{format:?}: {ffmpeg_format:?}");
                }
            }
        }
    }
}

// Test: RUST_BACKTRACE=1 cargo test --package blaze-rs --lib --all-features -- image::encdec::test::test --exact --nocapture