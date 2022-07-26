use std::{path::Path, ffi::{CString, CStr}};
use ffmpeg_next::{Frame};
use ffmpeg_sys_next::{avformat_alloc_context, av_guess_format};

use super::enc::ImageRead;

pub struct ImageWrite {

}

impl ImageWrite {
    pub fn new (frame: Frame, out: impl AsRef<Path>) {
        let mut format_ctx = unsafe { &mut *avformat_alloc_context() };
        let oformat = CString::new(out.as_ref().to_str().unwrap().split('.').last().unwrap()).unwrap();
        
        let a  = todo!();

        unsafe { format_ctx.oformat = av_guess_format(core::ptr::null(), oformat.as_ptr(), core::ptr::null()) };
        //format_ctx.iformat = 0;
    }
}

#[test]
fn test () {
    let read = ImageRead::new("tests/test.png").unwrap().read_frame().unwrap();
    let write = ImageWrite::new(read, "tests/test_out.png");
}