use std::{marker::PhantomData, mem::MaybeUninit, ptr::addr_of};
use crate::{prelude::*, image::{channel::{RawPixel, FromPrimitive}, RawImage, ChannelOrder}, event::WaitList, memobj::{IntoSlice2D}};

#[repr(transparent)]
pub struct FillImage<'dst> {
    event: RawEvent,
    dst: PhantomData<&'dst mut RawImage>
}

impl<'dst> FillImage<'dst> {
    #[inline]
    pub unsafe fn new<P: RawPixel> (dst: &'dst mut RawImage, color: &P, slice: impl IntoSlice2D, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> where f32: FromPrimitive<P::Subpixel> {
        if let Some(slice) = slice.into_slice(dst.width()?, dst.height()?) {
            let [origin, region] = slice.raw_parts();
            let color = Self::get_color(color);
            let event = dst.fill(color.as_ptr().cast(), origin, region, queue, wait)?;

            return Ok(Self {
                event,
                dst: PhantomData
            })
        }

        todo!()
    }

    fn get_color<P: RawPixel> (color: &P) -> [MaybeUninit<u8>; 16] where f32: FromPrimitive<P::Subpixel> {
        let mut result = MaybeUninit::uninit_array::<16>();
        let ptr = result.as_mut_ptr();

        #[cfg(feature = "cl2")]
        if P::ORDER == ChannelOrder::Depth {
            let value = f32::from_primitive(color.to_luma()[0]);
            unsafe { core::ptr::copy_nonoverlapping(addr_of!(value), ptr as *mut f32, 1) };
            return result
        }

        if P::TY.is_norm() {
            let channels = color.channels();
            unsafe { core::ptr::copy_nonoverlapping(channels.as_ptr(), ptr as *mut P::Subpixel, channels.len()) }
            return result
        }

        let color = color.to_rgba();
        let ptr = ptr as *mut f32;

        let channels = color.0.into_iter().map(f32::from_primitive).enumerate();
        for (i, v) in channels {
            unsafe { ptr.add(i).write(v) }
        }

        return result
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