use std::{marker::PhantomData, mem::MaybeUninit, ptr::addr_of};
use image::Pixel;

use crate::{prelude::*, image::{channel::{RawPixel, FromPrimitive}, RawImage, IntoSlice, ChannelOrder}, event::WaitList};

#[repr(transparent)]
pub struct FillImage<'dst> {
    event: RawEvent,
    dst: PhantomData<&'dst mut RawImage>
}

impl<'dst> FillImage<'dst> {
    #[inline]
    pub unsafe fn new<P: RawPixel> (dst: &'dst mut RawImage, color: &P, slice: impl IntoSlice<2>, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> where f32: FromPrimitive<P::Subpixel> {
        let color = Self::get_color(color);
        let slice = slice.into_slice([dst.width()?, dst.height()?]);
        let event = dst.fill(color.as_ptr().cast(), slice, queue, wait)?;

        Ok(Self {
            event,
            dst: PhantomData
        })
    }

    fn get_color<P: RawPixel> (color: &P) -> MaybeUninit<[u8; 16]> where f32: FromPrimitive<P::Subpixel> {
        let mut result = MaybeUninit::uninit();
        let ptr = result.as_mut_ptr();

        #[cfg(feature = "cl2")]
        if P::ORDER == ChannelOrder::Depth {
            let value = f32::from_primitive(color.channels()[0]);
            unsafe { core::ptr::copy_nonoverlapping(addr_of!(value).cast(), ptr as *mut u8, core::mem::size_of::<f32>()) };
            return result
        }

        if P::TY.is_norm() {
            let channels = color.channels();
            unsafe { core::ptr::copy_nonoverlapping(channels.as_ptr(), ptr as *mut P::Subpixel, channels.len()) }
            return result
        }

        let ptr = ptr as *mut P::Subpixel;
        todo!()
        
        unsafe {
            let count = <P as Pixel>::CHANNEL_COUNT as usize;
            core::ptr::write_bytes((ptr).add(count), 0, 4 - count);
        }

        let channels = color.channels().into_iter().copied().map(f32::from_primitive).enumerate();
        for (i, v) in channels {
            unsafe { ptr.add(i).write(v) }
        }

        todo!()
    }
}

impl<'dst> Event for FillImage<'dst> {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) }
        Ok(())
    }
}