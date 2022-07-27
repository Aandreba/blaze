use std::{alloc::{GlobalAlloc, Allocator}, ptr::{NonNull, addr_of_mut}};
use ffmpeg_sys_next::{av_malloc, av_free, AVFrame, av_frame_alloc, av_image_fill_arrays, av_image_get_linesize};
use crate::{buffer::rect::Rect2D, image::channel::RawPixel};
use super::error::{Error, ErrorKind};

pub type FfmpegBox<T> = Box<T, Ffmpeg>;
pub type FfmpegVec<T> = Vec<T, Ffmpeg>;

pub struct Ffmpeg;

unsafe impl Allocator for Ffmpeg {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe {
            if layout.size() == 0 {
                let slice = core::slice::from_raw_parts_mut(core::ptr::from_exposed_addr_mut(layout.align()), layout.size());
                return Ok(NonNull::new(slice).unwrap_unchecked());
            }

            match NonNull::new(self.alloc(layout)) {
                Some(x) => {
                    let slice = core::slice::from_raw_parts_mut(x.as_ptr(), layout.size());
                    Ok(NonNull::new_unchecked(slice))
                },

                None => Err(std::alloc::AllocError)
            }
        }
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        if layout.size() == 0 { return; }
        self.dealloc(ptr.as_ptr(), layout)
    }
}

unsafe impl GlobalAlloc for Ffmpeg {
    #[inline(always)]
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        av_malloc(layout.size()).cast()
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: std::alloc::Layout) {
        av_free(ptr.cast())
    }
}

#[inline(always)]
pub fn new_frame () -> FfmpegBox<AVFrame> {
    unsafe {
        let alloc = av_frame_alloc();
        assert!(!alloc.is_null());
        Box::from_raw_in(alloc, Ffmpeg)
    }
}

pub fn frame_to_rect<P: RawPixel> (frame: &AVFrame) -> super::error::Result<Rect2D<P, Ffmpeg>> {
    let mut ffmpeg_pixel = P::FFMPEG.ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid pixel format"))?;
    ffmpeg_pixel.update_id();

    let mut rect = Rect2D::<P, _>::new_uninit_in(
        usize::try_from(frame.width).unwrap(),
        usize::try_from(frame.width).unwrap(),
        Ffmpeg
    ).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid size"))?;

    let mut ptr = rect.as_mut_ptr() as *mut u8;
    let mut linesize = unsafe {
        av_image_get_linesize(ffmpeg_pixel.id, frame.width, 0)
    };

    unsafe {
        ffmpeg_tri!(av_image_fill_arrays (
            addr_of_mut!(ptr),
            addr_of_mut!(linesize),
            frame.data[0],
            ffmpeg_pixel.id,
            frame.width,
            frame.height,
            0
        ));

        Ok(rect.assume_init())
    }
}