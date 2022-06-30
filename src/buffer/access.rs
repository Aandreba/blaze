use std::{marker::PhantomData, ptr::{NonNull}, ops::{RangeBounds, Deref, DerefMut}, ffi::c_void, sync::Arc};
use parking_lot::{FairMutex};
use crate::{context::{Context, Global, RawContext}, event::{RawEvent, WaitList}};
use crate::core::*;
use crate::buffer::{flags::{FullMemFlags, HostPtr, MemAccess}, events::{ReadBufferEvent, WriteBufferEvent, ReadBufferInto, write_from_static, write_from_ptr}, manager::AccessManager, RawBuffer};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

pub trait MemObject<T: Copy + Unpin, C: Context>: AsRef<RawBuffer> + AsMut<RawBuffer> {
    const ACCESS : MemAccess;

    fn context (&self) -> &C;
    fn access_mananer (&self) -> Arc<FairMutex<AccessManager>>;

    #[inline(always)]
    fn len (&self) -> Result<usize> {
        let bytes = self.size()?;
        Ok(bytes / core::mem::size_of::<T>())
    }

    #[inline(always)]
    fn size (&self) -> Result<usize> {
        self.as_ref().size()
    }

    #[inline(always)]
    fn host_ptr (&self) -> Result<Option<NonNull<c_void>>> {
        self.as_ref().host_ptr()
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    fn map_count (&self) -> Result<u32> {
        self.as_ref().map_count()
    }

    /// Return _memobj_ reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    fn reference_count (&self) -> Result<u32> {
        self.as_ref().reference_count()
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    fn raw_context (&self) -> Result<RawContext> {
        self.as_ref().context()
    }

    #[inline(always)]
    fn offset (&self) -> Result<usize> {
        self.as_ref().offset()
    }

    #[inline(always)]
    fn read_all (&self, wait: impl Into<WaitList>) -> Result<ReadBufferEvent<T>> {
        self.read(.., wait)
    }

    #[inline(always)]
    fn read (&self, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<ReadBufferEvent<T>> {
        unsafe { ReadBufferEvent::new(self.as_ref(), range, self.context().next_queue(), wait) }
    }

    #[inline(always)]
    fn read_into<P: DerefMut<Target = [T]>> (&self, dst: P, offset: usize, wait: impl Into<WaitList>) -> Result<ReadBufferInto<T, P>> {
        unsafe { ReadBufferInto::new(self.as_ref(), dst, offset, self.context().next_queue(), wait)  }
    }

    #[inline(always)]
    fn write<P: Deref<Target = [T]>> (&mut self, src: P, offset: usize, wait: impl Into<WaitList>) -> Result<WriteBufferEvent<T, P>> {
        let queue = self.context().next_queue().clone();
        unsafe { WriteBufferEvent::new(src, self.as_mut(), offset, &queue, wait) }
    }

    #[inline(always)]
    fn write_static (&mut self, src: &'static [T], offset: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let queue = self.context().next_queue().clone();
        unsafe { write_from_static(src, self.as_mut(), offset, &queue, wait) }
    }

    #[inline(always)]
    unsafe fn write_ptr (&mut self, src: *const T, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let queue = self.context().next_queue().clone();
        write_from_ptr(src, self.as_mut(), range, &queue, wait)
    }
}

macro_rules! impl_buffer {
    ($($access:expr => $ident:ident),+) => {
        $(
            pub struct $ident<T: Copy, C: Context = Global> {
                inner: RawBuffer, 
                manager: Arc<FairMutex<AccessManager>>,
                ctx: C,
                phtm: PhantomData<T>
            }
            
            impl<T: Copy> $ident<T> {
                #[inline(always)]
                pub fn new (v: &[T], alloc: bool) -> Result<Self> {
                    Self::new_in(v, alloc, Global)
                }
            
                #[inline(always)]
                pub unsafe fn uninit (len: usize, alloc: bool) -> Result<Self> {
                    Self::uninit_in(len, alloc, Global)
                }
            
                #[inline(always)]
                pub fn create (len: usize, host: HostPtr, host_ptr: Option<NonNull<T>>) -> Result<Self> {
                    Self::create_in(len, host, host_ptr, Global)
                }
            }
            
            impl<T: Copy, C: Context> $ident<T, C> {
                #[inline]
                pub fn new_in (v: &[T], alloc: bool, ctx: C) -> Result<Self> {
                    let host = HostPtr::new(alloc, true);
                    Self::create_in(v.len(), host, NonNull::new(v.as_ptr() as *mut _), ctx)
                }
            
                #[inline(always)]
                pub unsafe fn uninit_in (len: usize, alloc: bool, ctx: C) -> Result<Self> {
                    let host = HostPtr::new(alloc, false);
                    Self::create_in(len, host, None, ctx)
                }
            
                #[inline]
                pub fn create_in (len: usize, host: HostPtr, host_ptr: Option<NonNull<T>>, ctx: C) -> Result<Self> {
                    let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
                    let inner = RawBuffer::new(size, FullMemFlags::new($access, host), host_ptr, ctx.raw_context())?;
            
                    Ok(Self {
                        inner,
                        manager: Arc::new(FairMutex::new(AccessManager::None)),
                        ctx,
                        phtm: PhantomData
                    })
                }
            }

            impl<T: Copy + Unpin, C: Context> MemObject<T, C> for $ident<T, C> {
                const ACCESS : MemAccess = $access;

                #[inline(always)]
                fn access_mananer (&self) -> Arc<FairMutex<AccessManager>> {
                    self.manager.clone()
                }
            
                /// Return context specified when memory object is created.
                #[inline(always)]
                fn context (&self) -> &C {
                    &self.ctx
                }
            }

            impl<T: Copy + Unpin, C: Context> AsRef<RawBuffer> for $ident<T, C> {
                #[inline(always)]
                fn as_ref (&self) -> &RawBuffer {
                    &self.inner
                }
            }

            impl<T: Copy + Unpin, C: Context> AsMut<RawBuffer> for $ident<T, C> {
                #[inline(always)]
                fn as_mut (&mut self) -> &mut RawBuffer {
                    &mut self.inner
                }
            }
        )+
    };
}

impl_buffer! {
    MemAccess::READ_WRITE => Buffer,
    MemAccess::READ_ONLY => ReadBuffer,
    MemAccess::WRITE_ONLY => WriteBuffer
}