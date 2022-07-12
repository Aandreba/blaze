use std::{ptr::NonNull, ops::{Deref}};
use crate::{prelude::*, buffer::{RawBuffer, IntoRange}, event::WaitList, memobj::{MapBox, Map, MemObject, MapRefBox, MapMutBox, MapMut}};

pub struct MapBuffer<T, C: Context> {
    event: RawEvent,
    mem: MemObject,
    len: usize,
    ptr: NonNull<T>,
    ctx: C
}

impl<T, C: Context> MapBuffer<T, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: RawBuffer, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(&src)?;
        let (ptr, event) = src.map_read_write(range, ctx.next_queue(), wait)?;
        let ptr : NonNull<T> = NonNull::new(ptr).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            mem: src.into(),
            len: range.cb / core::mem::size_of::<T>()
        })
    }
}

impl<T, C: Context> Event for MapBuffer<T, C> {
    type Output = MapBox<T, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        
        unsafe {
            let ptr = core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len);
            Ok(MapBox::from_raw_in(ptr, Map::new_in(self.ctx, self.mem)))
        }
    }
}

pub struct MapRefBuffer<'a, T, C: Context> {
    event: RawEvent,
    mem: &'a MemObject,
    len: usize,
    ptr: NonNull<T>,
    ctx: C
}

impl<'a, T, C: Context> MapRefBuffer<'a, T, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: &'a RawBuffer, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(&src)?;
        let (ptr, event) : (*const T, _) = src.map_read(range, ctx.next_queue(), wait)?;
        let ptr = NonNull::new(ptr as *mut _).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            mem: src.deref(),
            len: range.cb / core::mem::size_of::<T>()
        })
    }
}

impl<'a, T, C: Context> Event for MapRefBuffer<'a, T, C> {
    type Output = MapRefBox<'a, T, C>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        
        unsafe {
            Ok(MapRefBox::from_raw_parts_in(self.mem, self.ptr.as_ptr(), self.len, self.ctx))
        }
    }
}

pub struct MapMutBuffer<'a, T, C: Context> {
    event: RawEvent,
    mem: &'a mut MemObject,
    len: usize,
    ptr: NonNull<T>,
    ctx: C
}

impl<'a, T, C: Context> MapMutBuffer<'a, T, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: &'a mut RawBuffer, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(&src)?;
        let (ptr, event) = src.map_read_write(range, ctx.next_queue(), wait)?;
        let ptr : NonNull<T> = NonNull::new(ptr).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            mem: src,
            len: range.cb / core::mem::size_of::<T>()
        })
    }
}

impl<'a, T, C: Context> Event for MapMutBuffer<'a, T, C> {
    type Output = MapMutBox<'a, T, C>;

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