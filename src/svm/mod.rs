flat_mod!(flags, utils);

#[cfg_attr(docsrs, doc(cfg(feature = "atomics")))]
#[cfg(feature = "atomics")]
pub mod atomics;

use std::{alloc::{Layout, Allocator, GlobalAlloc}, ptr::{NonNull, addr_of_mut}, ffi::c_void};
use opencl_sys::{clSVMAlloc, clSVMFree, clEnqueueSVMFree, clEnqueueSVMMap, CL_TRUE, CL_MAP_READ, CL_MAP_WRITE, cl_map_flags, CL_FALSE, clEnqueueSVMUnmap};
use crate::{context::{Context, Global}, event::{RawEvent, WaitList}, core::Result, prelude::{Error, ErrorType, device::SvmCapability}, buffer::flags::MemAccess};

#[derive(Clone, Copy)]
pub struct Svm<C: Context = Global> {
    ctx: C,
    coarse: bool
}

impl Svm {
    #[inline(always)]
    pub const fn new (coarse: bool) -> Self {
        Self::new_in(Global, coarse)
    }

    #[inline(always)]
    pub fn try_default () -> Result<Self> {
        Self::try_default_in(Global)
    }
}

impl<C: Context> Svm<C> {
    #[inline(always)]
    pub const fn new_in (ctx: C, coarse: bool) -> Self {
        Self {
            ctx,
            coarse
        }
    }

    pub fn try_default_in (ctx: C) -> Result<Self> {
        let mut fine = true;

        for queue in ctx.queues() {
            let device = queue.device()?;
            if !device.svm_capabilities()?.intersects(SvmCapability::FINE_GRAIN_BUFFER | SvmCapability::FINE_GRAIN_SYSTEM) {
                fine = false;
                break
            }
        }

        Ok(Self::new_in(ctx, !fine))
    }

    #[inline(always)]
    pub const fn is_coarse (&self) -> bool {
        self.coarse
    }

    #[inline]
    pub unsafe fn alloc_with_flags (&self, flags: SvmFlags, layout: Layout) -> Result<*mut u8> {
        #[cfg(debug_assertions)]
        if self.coarse && flags.utils.is_some() {
            return Err(Error::new(ErrorType::InvalidValue, "SVM allocator marked as coarse-grained, but added fine-grained flags"));
        }

        let align = u32::try_from(layout.align()).unwrap();
        let ptr = clSVMAlloc(self.ctx.id(), flags.to_bits(), layout.size(), align);

        if self.coarse { self.map_blocking::<WaitList, {CL_MAP_READ | CL_MAP_WRITE}>(ptr, layout.size(), WaitList::EMPTY)?; }
        Ok(ptr.cast())
    }

    #[inline(always)]
    pub(crate) unsafe fn map<W: Into<WaitList>, const MASK: cl_map_flags> (&self, ptr: *mut c_void, size: usize, wait: W) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueSVMMap(self.ctx.next_queue().id(), CL_FALSE, MASK, ptr, size, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));

        Ok(RawEvent::from_id(evt).unwrap())
    }

    #[inline(always)]
    pub(crate) unsafe fn map_blocking<W: Into<WaitList>, const MASK: cl_map_flags> (&self, ptr: *mut c_void, size: usize, wait: W) -> Result<()> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        tri!(clEnqueueSVMMap(self.ctx.next_queue().id(), CL_TRUE, MASK, ptr, size, num_events_in_wait_list, event_wait_list, core::ptr::null_mut()));
        Ok(())
    }

    pub(crate) unsafe fn unmap (&self, ptr: *mut c_void, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueSVMUnmap(self.ctx.next_queue().id(), ptr, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        
        Ok(RawEvent::from_id(evt).unwrap())
    }

    #[inline(always)]
    pub unsafe fn free (&self, ptr: *mut u8) {
        clSVMFree(self.ctx.id(), ptr.cast())
    }

    #[inline(always)]
    pub unsafe fn enqueue_free (&self, ptrs: &[*const c_void], wait: impl Into<WaitList>) -> Result<RawEvent> {
        let len = u32::try_from(ptrs.len()).expect("Too many pointers");
        
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueSVMFree(self.ctx.next_queue().id(), len, ptrs.as_ptr(), None, core::ptr::null_mut(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
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
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: Layout) {
        self.free(ptr.as_ptr().cast())
    }
}

unsafe impl<C: Context> GlobalAlloc for Svm<C> {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        const DEFAULT_FINE : SvmFlags = SvmFlags::const_new(MemAccess::READ_WRITE, Some(SvmUtilsFlags::FineGrain));
        const DEFAULT_COARSE : SvmFlags = SvmFlags::const_new(MemAccess::READ_WRITE, None);

        let flags = match self.coarse {
            true => DEFAULT_COARSE,
            false => DEFAULT_FINE
        };

        self.alloc_with_flags(flags, layout).unwrap()
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.free(ptr.cast())
    }
}

impl Default for Svm {
    #[inline(always)]
    fn default() -> Self {
        Self::try_default().unwrap()
    }
}