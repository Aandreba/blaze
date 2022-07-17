use std::{alloc::Allocator, ptr::addr_of_mut, ops::{Deref}, borrow::{Borrow}};
use opencl_sys::clEnqueueUnmapMemObject;
use crate::{prelude::*, event::WaitList};
use super::{AsMem, AsMutMem};

pub type MapMutBox<T, D, C = Global> = Box<[T], MapMut<D, C>>;

pub struct Map<D: AsMem, C: Context = Global> (D, C);

impl<D: AsMem, C: Context> Map<D, C> {
    #[inline(always)]
    pub(crate) const fn new_in (ctx: C, buff: D) -> Self {
        Self(buff, ctx)
    }

    pub unsafe fn unmap (&self, ptr: *const u8, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueUnmapMemObject(self.1.next_queue().id(), self.0.as_mem().id(), ptr as *mut _, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
    }
}

unsafe impl<D: AsMem, C: Context> Allocator for Map<D, C> {
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
pub struct MapMut<D: AsMutMem, C: Context = Global> (D, C);

impl<D: AsMutMem, C: Context> MapMut<D, C> {
    #[inline(always)]
    pub(crate) fn new_in (ctx: C, buff: D) -> Self {
        Self(buff, ctx)
    }

    pub unsafe fn unmap (&self, ptr: *const u8, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueUnmapMemObject(self.1.next_queue().id(), self.0.as_mem().id(), ptr as *mut _, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
    }
}

unsafe impl<D: AsMutMem, C: Context> Allocator for MapMut<D, C> {
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
pub struct MapBox<T, D: AsMem, C: Context = Global> (Box<[T], Map<D, C>>);

impl<T, D: AsMem, C: Context> MapBox<T, D, C> {
    #[inline]
    pub(crate) unsafe fn from_raw_parts_in (mem: D, ptr: *mut T, len: usize, ctx: C) -> Self {
        let ptr = core::slice::from_raw_parts_mut(ptr, len);
        Self(Box::from_raw_in(ptr, Map::new_in(ctx, mem)))
    }
}

impl<T, D: AsMem, C: Context> Deref for MapBox<T, D, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, D: AsMem, C: Context> Borrow<Box<[T], Map<D, C>>> for MapBox<T, D, C> {
    #[inline(always)]
    fn borrow(&self) -> &Box<[T], Map<D, C>> {
        &self.0
    }
}

impl<T, D: AsMem, C: Context> AsRef<Box<[T], Map<D, C>>> for MapBox<T, D, C> {
    #[inline(always)]
    fn as_ref(&self) -> &Box<[T], Map<D, C>> {
        &self.0
    }
}