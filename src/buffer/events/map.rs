use std::{ptr::NonNull, ops::{Deref, DerefMut}};
use crate::{prelude::*, buffer::{IntoRange, Buffer}, event::WaitList, memobj::{MapBox, MapMutBox, MapMut}};

pub struct MapBuffer<T, D, C: Context> {
    event: RawEvent,
    mem: D,
    len: usize,
    ptr: NonNull<T>,
    ctx: C
}

impl<T: 'static + Copy, D: Deref<Target = Buffer<T, C>>, C: 'static + Context> MapBuffer<T, D, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: D, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(&src)?;
        let (ptr, event) : (*const T, _) = src.map_read_in(range, ctx.next_queue(), wait)?;
        let ptr = NonNull::new(ptr as *mut _).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            mem: src,
            len: range.cb / core::mem::size_of::<T>()
        })
    }
}

impl<T: 'static + Copy, D: Deref<Target = Buffer<T, C>>, C: 'static + Context> Event for MapBuffer<T, D, C> {
    type Output = MapBox<T, D, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        
        unsafe {
            Ok(MapBox::from_raw_parts_in(self.mem, self.ptr.as_ptr(), self.len, self.ctx))
        }
    }
}

pub struct MapMutBuffer<T, D, C: Context> {
    event: RawEvent,
    mem: D,
    len: usize,
    ptr: NonNull<T>,
    ctx: C
}

impl<T: 'static + Copy, D: DerefMut<Target = Buffer<T, C>>, C: 'static + Context> MapMutBuffer<T, D, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: D, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(&src)?;
        let (ptr, event) = src.map_read_write_in(range, ctx.next_queue(), wait)?;
        let ptr : NonNull<T> = NonNull::new(ptr).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            mem: src,
            len: range.cb / core::mem::size_of::<T>()
        })
    }
}

impl<T: 'static + Copy, D: DerefMut<Target = Buffer<T, C>>, C: 'static + Context> Event for MapMutBuffer<T, D, C> {
    type Output = MapMutBox<T, D, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        
        unsafe {
            let ptr = core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len);
            Ok(MapMutBox::from_raw_in(ptr, MapMut::new_in(self.ctx, self.mem)))
        }
    }
}