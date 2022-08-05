use std::{ops::{Deref, DerefMut}, ptr::{addr_of_mut, NonNull, addr_of}, fmt::Debug};
use opencl_sys::*;
use crate::{prelude::*, buffer::{IntoRange, BufferRange}};

pub struct MapBuffer<T, S> {
    evt: RawEvent,
    ptr: NonNull<[T]>,
    src: S
}

impl<T: 'static + Copy, S: Deref<Target = Buffer<T, C>>, C: 'static + Context> MapBuffer<T, S> {
    #[inline(always)]
    pub fn new<R: IntoRange, W: Into<WaitList>> (src: S, range: R, wait: W) -> Result<Self> {
        Self::new_inner::<R, W, CL_MAP_READ>(src, range, wait)
    }

    fn new_inner<R: IntoRange, W: Into<WaitList>, const FLAG: cl_mem_flags> (src: S, range: R, wait: W) -> Result<Self> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        let BufferRange { offset, cb } = range.into_range::<T>(&src)?;

        let mut err = 0;
        let mut evt = core::ptr::null_mut();

        unsafe {
            let ptr = clEnqueueMapBuffer(src.ctx.next_queue().id(), src.id(), CL_FALSE, FLAG, offset, cb, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt), addr_of_mut!(err));
    
            if err != 0 {
                return Err(Error::from(err))
            }
    
            let evt = RawEvent::from_id(evt).unwrap();
            let ptr = {
                let slice = core::slice::from_raw_parts_mut(ptr as *mut T, cb / core::mem::size_of::<T>());
                NonNull::new(slice)
            }.ok_or_else(|| Error::from_type(crate::prelude::ErrorType::InvalidValue))?;
    
            Ok(Self { evt, src, ptr })
        }
    }
}

impl<T: 'static + Copy, S: Deref<Target = Buffer<T, C>>, C: 'static + Context> Event for MapBuffer<T, S> {
    type Output = MapBufferGuard<T, S, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.evt
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        Ok(MapBufferGuard::new(self.ptr, self.src))
    }
}

#[repr(transparent)]
pub struct MapBufferMut<T, S> (MapBuffer<T, S>);

impl<T: 'static + Copy, S: DerefMut<Target = Buffer<T, C>>, C: 'static + Context> MapBufferMut<T, S> {
    #[inline(always)]
    pub fn new<R: IntoRange, W: Into<WaitList>> (src: S, range: R, wait: W) -> Result<Self> {
        MapBuffer::new_inner::<R, W, {CL_MAP_READ | CL_MAP_WRITE}>(src, range, wait).map(Self)
    }
}

impl<T: 'static + Copy, S: DerefMut<Target = Buffer<T, C>>, C: 'static + Context> Event for MapBufferMut<T, S> {
    type Output = MapBufferMutGuard<T, S, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        self.0.consume(err).map(MapBufferMutGuard)
    }
}

/* GUARDS */

/// Guard for mapped memory object region
pub struct MapBufferGuard<T: Copy, S: Deref<Target = Buffer<T, C>>, C: Context = Global> {
    ptr: NonNull<[T]>,
    src: S
}

impl<T: Copy, S: Deref<Target = Buffer<T, C>>, C: Context> MapBufferGuard<T, S, C> {
    #[inline(always)]
    pub(crate) const fn new (ptr: NonNull<[T]>, src: S) -> Self {
        Self { ptr, src }
    }
}

impl<T: Copy, S: Deref<Target = Buffer<T, C>>, C: Context> Deref for MapBufferGuard<T, S, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Debug + Copy, S: Deref<Target = Buffer<T, C>>, C: Context> Debug for MapBufferGuard<T, S, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T: Copy, S: Deref<Target = Buffer<T, C>>, C: Context> Drop for MapBufferGuard<T, S, C> {
    #[inline(always)]
    fn drop(&mut self) {        
        let mut evt = core::ptr::null_mut();

        unsafe {
            tri_panic! {
                clEnqueueUnmapMemObject(self.src.ctx.next_queue().id(), self.src.id(), self.ptr.as_ptr().cast(), 0, core::ptr::null(), addr_of_mut!(evt));
                clWaitForEvents(1, addr_of!(evt))
            }
        }
    }
}

unsafe impl<T: Copy, S: Send + Deref<Target = Buffer<T, C>>, C: Context> Send for MapBufferGuard<T, S, C> {}
unsafe impl<T: Copy, S: Sync + Deref<Target = Buffer<T, C>>, C: Context> Sync for MapBufferGuard<T, S, C> {}

/// Guard for mutably mapped memory object region
#[repr(transparent)]
pub struct MapBufferMutGuard<T: Copy, S: DerefMut<Target = Buffer<T, C>>, C: Context> (MapBufferGuard<T, S, C>);

impl<T: Copy, S: DerefMut<Target = Buffer<T, C>>, C: Context> Deref for MapBufferMutGuard<T, S, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: Copy, S: DerefMut<Target = Buffer<T, C>>, C: Context> DerefMut for MapBufferMutGuard<T, S, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.ptr.as_mut() }
    }
}

impl<T: Debug + Copy, S: DerefMut<Target = Buffer<T, C>>, C: Context> Debug for MapBufferMutGuard<T, S, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}