use std::{ops::{Deref, DerefMut}, ptr::{addr_of_mut, NonNull, addr_of}, mem::ManuallyDrop, fmt::Debug};
use crate::{prelude::*, image::{Image2D, channel::RawPixel}, memobj::IntoSlice2D};
use opencl_sys::*;

pub struct MapImage2D<T, S> {
    evt: RawEvent,
    rect: ManuallyDrop<Rect2D<T>>,
    src: S
}

impl<T: 'static + RawPixel, S: Deref<Target = Image2D<T, C>>, C: 'static + Context> MapImage2D<T, S> {
    #[inline(always)]
    pub fn new<R: IntoSlice2D, W: Into<WaitList>> (src: S, slice: R, wait: W) -> Result<Self> {
        Self::new_inner::<R, W, CL_MAP_READ>(src, slice, wait)
    }

    fn new_inner<R: IntoSlice2D, W: Into<WaitList>, const FLAG: cl_mem_flags> (src: S, slice: R, wait: W) -> Result<Self> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        let slice = slice.into_slice(src.width()?, src.height()?).ok_or_else(|| Error::from_type(ErrorType::InvalidValue))?;
        let [origin, region] = slice.raw_parts_buffer::<T>();

        let mut err = 0;
        let mut evt = core::ptr::null_mut();
        let mut row_pitch = 0;

        unsafe {
            let ptr = clEnqueueMapImage(
                src.ctx.next_queue().id(), src.id(), CL_FALSE, FLAG,
                origin.as_ptr(), region.as_ptr(), addr_of_mut!(row_pitch), core::ptr::null_mut(),
                num_events_in_wait_list, event_wait_list,
                addr_of_mut!(evt), addr_of_mut!(err)
            );
    
            if err != 0 {
                return Err(Error::from(err))
            }
    
            let evt = RawEvent::from_id(evt).unwrap();
            let rect = unsafe {
                let ptr = NonNull::new(ptr).ok_or_else(|| Error::from_type(crate::prelude::ErrorType::InvalidValue))?;
                ManuallyDrop::new(Rect2D::from_raw_parts(ptr.cast(), slice.region_x, slice.region_y))
            };
    
            Ok(Self { evt, src, rect })
        }
    }
}

impl<T: 'static + RawPixel, S: Deref<Target = Image2D<T, C>>, C: 'static + Context> Event for MapImage2D<T, S> {
    type Output = MapImage2DGuard<T, S, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.evt
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        Ok(MapImage2DGuard::new(self.rect, self.src))
    }
}

#[repr(transparent)]
pub struct MapImage2DMut<T, S> (MapImage2D<T, S>);

impl<T: 'static + RawPixel, S: DerefMut<Target = Image2D<T, C>>, C: 'static + Context> MapImage2DMut<T, S> {
    #[inline(always)]
    pub fn new<R: IntoSlice2D, W: Into<WaitList>> (src: S, slice: R, wait: W) -> Result<Self> {
        MapImage2D::new_inner::<R, W, {CL_MAP_READ | CL_MAP_WRITE}>(src, slice, wait).map(Self)
    }
}

impl<T: 'static + RawPixel, S: DerefMut<Target = Image2D<T, C>>, C: 'static + Context> Event for MapImage2DMut<T, S> {
    type Output = MapImage2DMutGuard<T, S, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        self.0.consume(err).map(MapImage2DMutGuard)
    }
}

/* GUARDS */

/// Guard for mapped memory object region
pub struct MapImage2DGuard<T: RawPixel, S: Deref<Target = Image2D<T, C>>, C: Context = Global> {
    rect: ManuallyDrop<Rect2D<T>>,
    src: S
}

impl<T: RawPixel, S: Deref<Target = Image2D<T, C>>, C: Context> MapImage2DGuard<T, S, C> {
    #[inline(always)]
    pub(crate) const fn new (rect: ManuallyDrop<Rect2D<T>>, src: S) -> Self {
        Self { rect, src }
    }
}

impl<T: RawPixel, S: Deref<Target = Image2D<T, C>>, C: Context> Deref for MapImage2DGuard<T, S, C> {
    type Target = Rect2D<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.rect.deref()
    }
}

impl<T: Debug + RawPixel, S: Deref<Target = Image2D<T, C>>, C: Context> Debug for MapImage2DGuard<T, S, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T: RawPixel, S: Deref<Target = Image2D<T, C>>, C: Context> Drop for MapImage2DGuard<T, S, C> {
    #[inline(always)]
    fn drop(&mut self) {        
        let mut evt = core::ptr::null_mut();

        unsafe {
            tri_panic! {
                clEnqueueUnmapMemObject(self.src.ctx.next_queue().id(), self.src.id(), self.rect.as_mut_ptr().cast(), 0, core::ptr::null(), addr_of_mut!(evt));
                clWaitForEvents(1, addr_of!(evt))
            }
        }
    }
}

unsafe impl<T: RawPixel, S: Send + Deref<Target = Image2D<T, C>>, C: Context> Send for MapImage2DGuard<T, S, C> {}
unsafe impl<T: RawPixel, S: Sync + Deref<Target = Image2D<T, C>>, C: Context> Sync for MapImage2DGuard<T, S, C> {}

/// Guard for mutably mapped memory object region
#[repr(transparent)]
pub struct MapImage2DMutGuard<T: RawPixel, S: DerefMut<Target = Image2D<T, C>>, C: Context> (MapImage2DGuard<T, S, C>);

impl<T: RawPixel, S: DerefMut<Target = Image2D<T, C>>, C: Context> Deref for MapImage2DMutGuard<T, S, C> {
    type Target = Rect2D<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: RawPixel, S: DerefMut<Target = Image2D<T, C>>, C: Context> DerefMut for MapImage2DMutGuard<T, S, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.rect.deref_mut()
    }
}

impl<T: Debug + RawPixel, S: DerefMut<Target = Image2D<T, C>>, C: Context> Debug for MapImage2DMutGuard<T, S, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}