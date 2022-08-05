use std::ptr::NonNull;
use ffmpeg_sys_next::{*};

pub fn get_sws_context (src: AVPixelFormat, dst: AVPixelFormat, width: i32, height: i32) -> Option<NonNull<SwsContext>> {
    let inner = unsafe {
        sws_getContext(
            width,
            height,
            src,
            width,
            height,
            dst,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut()
        )
    };

    NonNull::new(inner)
}