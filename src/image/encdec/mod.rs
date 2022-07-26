pub mod enc;
pub mod dec;

/*pub fn ffmpeg_write<P: RawPixel> (rect: Rect2D<P>, format: ffmpeg_next::codec::Id, threads: Option<std::num::NonZeroUsize>) -> Result<(), ffmpeg_next::Error> {
    use ffmpeg_sys_next::{avcodec_alloc_context3, FF_THREAD_SLICE};

    let enc_codec = ffmpeg_next::codec::encoder::find(format).ok_or(ffmpeg_next::Error::EncoderNotFound)?;    
    let mut enc_ctx = unsafe {
        let ptr = avcodec_alloc_context3(enc_codec.as_ptr());
        if ptr.is_null() {
            return Err(ffmpeg_next::Error::Unknown)
        }

        ffmpeg_next::codec::Context::wrap(ptr, None)
    };

    let enc_ctx_ref = unsafe { &mut *enc_ctx.as_mut_ptr() };
    enc_ctx_ref.width = i32::try_from(rect.width()).map_err(|_| ffmpeg_next::Error::InvalidData)?;
    enc_ctx_ref.height = i32::try_from(rect.height()).map_err(|_| ffmpeg_next::Error::InvalidData)?;
    enc_ctx_ref.thread_type = FF_THREAD_SLICE;
    enc_ctx_ref.thread_count = match threads {
        Some(x) => i32::try_from(x.get()).map_err(|_| ffmpeg_next::Error::InvalidData)?,
        None => match std::thread::available_parallelism() {
            Ok(x) => i32::try_from(x.get()).map_err(|_| ffmpeg_next::Error::InvalidData)?,
            Err(_) => {
                #[cfg(debug_assertions)]
                eprintln!("Error obtaining available parallelism");

                1
            }
        }
    };

    todo!()
}*/