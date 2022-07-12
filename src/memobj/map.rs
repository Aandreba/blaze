use std::{alloc::Allocator, ptr::addr_of_mut, ops::Deref, mem::ManuallyDrop};
use opencl_sys::clEnqueueUnmapMemObject;
use crate::{prelude::*, event::WaitList};
use super::MemObject;

pub type MapBox<T, C = Global> = Box<[T], Map<C>>;
pub type MapMutBox<'a, T, C = Global> = Box<[T], MapMut<'a, C>>;

pub struct Map<C: Context = Global> (MemObject, C);

impl<C: Context> Map<C> {
    #[inline(always)]
    pub(crate) const fn new_in (ctx: C, buff: MemObject) -> Self {
        Self(buff, ctx)
    }

    pub unsafe fn unmap (&self, ptr: *const u8, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueUnmapMemObject(self.1.next_queue().id(), self.0.id(), ptr as *mut _, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
    }
}

unsafe impl<C: Context> Allocator for Map<C> {
    #[inline(always)]
    fn allocate(&self, _layout: std::alloc::Layout) -> core::result::Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        Err(std::alloc::AllocError)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        self.unmap(ptr.as_ptr(), WaitList::EMPTY).unwrap().wait().unwrap();
    }
}

/// Mapped memory object by reference
pub struct MapRef<'a, C: Context = Global> (&'a MemObject, C);

impl<'a, C: Context> MapRef<'a, C> {
    #[inline(always)]
    pub const fn new_in (ctx: C, buff: &'a MemObject) -> Self {
        Self(buff, ctx)
    }

    pub unsafe fn unmap (&self, ptr: *const u8, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueUnmapMemObject(self.1.next_queue().id(), self.0.id(), ptr as *mut _, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
    }
}

unsafe impl<C: Context> Allocator for MapRef<'_, C> {
    #[inline(always)]
    fn allocate(&self, _layout: std::alloc::Layout) -> core::result::Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        Err(std::alloc::AllocError)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        self.unmap(ptr.as_ptr(), WaitList::EMPTY).unwrap().wait().unwrap();
    }
}

/// Mapped memory object by mutable reference.
pub struct MapMut<'a, C: Context = Global> (&'a mut MemObject, C);

impl<'a, C: Context> MapMut<'a, C> {
    #[inline(always)]
    pub(crate) fn new_in (ctx: C, buff: &'a mut MemObject) -> Self {
        Self(buff, ctx)
    }

    pub unsafe fn unmap (&self, ptr: *const u8, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueUnmapMemObject(self.1.next_queue().id(), self.0.id(), ptr as *mut _, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
    }
}

unsafe impl<C: Context> Allocator for MapMut<'_, C> {
    #[inline(always)]
    fn allocate(&self, _layout: std::alloc::Layout) -> core::result::Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        Err(std::alloc::AllocError)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        self.unmap(ptr.as_ptr(), WaitList::EMPTY).unwrap().wait().unwrap();
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MapRefBox<'a, T, C: Context = Global> (Box<[T], MapRef<'a, C>>);

impl<'a, T, C: Context> MapRefBox<'a, T, C> {
    #[inline]
    pub(crate) unsafe fn from_raw_parts_in (mem: &'a MemObject, ptr: *mut T, len: usize, ctx: C) -> Self {
        let ptr = core::slice::from_raw_parts_mut(ptr, len);
        Self(Box::from_raw_in(ptr, MapRef::new_in(ctx, mem)))
    }
}

impl<'a, T, C: Context> Deref for MapRefBox<'a, T, C> {
    type Target = Box<[T], MapRef<'a, C>>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use sealed::Sealed;

mod sealed {
    pub trait Sealed {}
}

pub trait MapBoxExt<T>: Sized + Sealed {
    unsafe fn unmap_wait (self, wait: impl Into<WaitList>) -> Result<RawEvent>;

    #[inline(always)]
    fn unmap (self) -> Result<RawEvent> {
        unsafe { self.unmap_wait(WaitList::EMPTY) }
    }
}

impl<T, C: Context> MapBoxExt<T> for MapBox<T, C> {
    #[inline]
    unsafe fn unmap_wait (self, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let this = ManuallyDrop::new(self);
        let alloc = Box::allocator(&this);
        alloc.unmap(this.as_ptr().cast(), wait)
    }
}

impl<T, C: Context> MapBoxExt<T> for MapRefBox<'_, T, C> {
    #[inline]
    unsafe fn unmap_wait (self, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let this = ManuallyDrop::new(self);
        let alloc = Box::allocator(&this);
        alloc.unmap(this.as_ptr().cast(), wait)
    }
}

impl<T, C: Context> MapBoxExt<T> for MapMutBox<'_, T, C> {
    #[inline]
    unsafe fn unmap_wait (self, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let this = ManuallyDrop::new(self);
        let alloc = Box::allocator(&this);
        alloc.unmap(this.as_ptr().cast(), wait)
    }
}

impl<T, C: Context> Sealed for MapBox<T, C> {}
impl<T, C: Context> Sealed for MapRefBox<'_, T, C> {}
impl<T, C: Context> Sealed for MapMutBox<'_, T, C> {}