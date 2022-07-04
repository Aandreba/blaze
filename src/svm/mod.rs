flat_mod!(flags, utils);

#[cfg_attr(docsrs, doc(cfg(feature = "atomics")))]
#[cfg(feature = "atomics")]
pub mod atomics;

use std::{alloc::{Layout, Allocator, GlobalAlloc}, ptr::NonNull};
use opencl_sys::{clSVMAlloc, clSVMFree};
use crate::{context::{Context, Global}};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Svm<C: Context = Global> (C);

impl Svm {
    #[inline(always)]
    pub const fn new () -> Self {
        Self::new_in(Global)
    }
}

impl<C: Context> Svm<C> {
    #[inline(always)]
    pub const fn new_in (ctx: C) -> Self {
        Self(ctx)
    }

    #[inline(always)]
    pub unsafe fn alloc_with_flags (&self, flags: SvmFlags, layout: Layout) -> *mut u8 {
        let align = u32::try_from(layout.align()).unwrap();
        clSVMAlloc(self.0.raw_context().id(), flags.to_bits(), layout.size(), align).cast()
    }

    #[inline(always)]
    pub unsafe fn free (&self, ptr: *mut u8) {
        clSVMFree(self.0.raw_context().id(), ptr.cast())
    }
}

unsafe impl<C: Context> Allocator for Svm<C> {
    fn allocate(&self, layout: Layout) -> core::result::Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        if layout.size() == 0 {
            let ptr : *mut [u8] = core::ptr::from_raw_parts_mut(core::ptr::invalid_mut(layout.align()), 0);
            return Ok(unsafe { NonNull::new_unchecked(ptr) });
        }

        let alloc : *mut [u8] = unsafe { core::ptr::from_raw_parts_mut(self.alloc(layout).cast(), layout.size()) };
        NonNull::new(alloc).ok_or(std::alloc::AllocError)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: Layout) {
        self.dealloc(ptr.as_ptr(), layout)
    }
}

unsafe impl<C: Context> GlobalAlloc for Svm<C> {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_with_flags(SvmFlags::default(), layout)
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.free(ptr.cast())
    }
}