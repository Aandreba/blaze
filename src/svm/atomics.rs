use std::{alloc::Layout, sync::atomic::*, ops::{Deref, DerefMut, Index, IndexMut}};
use crate::context::{Context, Global};
use super::{SvmBox, Svm, SvmUtilsFlags};
use crate::prelude::*;
use crate::buffer::flags::MemAccess;
use crate::svm::SvmFlags;
use crate::buffer::KernelPointer;
use blaze_proc::docfg;

macro_rules! impl_atomic {
    ($($len:literal in $ty:ty => $atomic:ty as $svm:ident),+) => {
        $(
            #[docfg(target_has_atomic = $len)]
            #[repr(transparent)]
            pub struct $svm<C: Context = Global> (SvmBox<[$ty], C>);

            #[cfg(target_has_atomic = $len)]
            impl $svm {
                #[inline(always)]
                pub fn new (v: &[$ty]) -> Self {
                    Self::new_in(v, Global)
                }
            }

            #[cfg(target_has_atomic = $len)]
            impl<C: Context> $svm<C> {
                pub fn new_in (v: &[$ty], ctx: C) -> Self {
                    let alloc = Svm::new_in(ctx, false);
                    let layout = Layout::array::<$ty>(v.len()).unwrap();
                    let boxed;

                    unsafe {
                        let ptr = alloc.alloc_with_flags(SvmFlags::new(MemAccess::default(), SvmUtilsFlags::Atomics), layout).unwrap();
                        let ptr : *mut [$ty] = core::ptr::from_raw_parts_mut(ptr.cast(), v.len());

                        assert!(!ptr.is_null());
                        boxed = SvmBox::from_raw_in(ptr, alloc);
                    }

                    unsafe { Self::from_box(boxed) }
                }
                
                #[inline(always)]
                pub const unsafe fn from_box (v: SvmBox<[$ty], C>) -> Self {
                    Self(v)
                }

                #[inline(always)]
                pub unsafe fn as_ptr (&self) -> *mut $ty {
                    self.0.as_ptr() as *mut _
                }
            }

            #[cfg(target_has_atomic = $len)]
            impl<C: Context> Deref for $svm<C> {
                type Target = [$atomic];

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    debug_assert_eq!(core::mem::align_of::<$atomic>(), core::mem::align_of::<$ty>());
                    let v = self.0.deref();

                    // SAFETY:
                    //  - the mutable reference guarantees unique ownership.
                    //  - the alignment of `$int_type` and `Self` is the
                    //    same, as promised by $cfg_align and verified above.
                    unsafe { & *(v as *const [$ty] as *const [$atomic]) }
                }
            }

            #[cfg(target_has_atomic = $len)]
            impl<C: Context> DerefMut for $svm<C> {
                #[inline(always)]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    debug_assert_eq!(core::mem::align_of::<$atomic>(), core::mem::align_of::<$ty>());
                    let v = self.0.deref_mut();

                    // SAFETY:
                    //  - the mutable reference guarantees unique ownership.
                    //  - the alignment of `$int_type` and `Self` is the
                    //    same, as promised by $cfg_align and verified above.
                    unsafe { &mut *(v as *mut [$ty] as *mut [$atomic]) }
                }
            }

            #[cfg(target_has_atomic = $len)]
            unsafe impl<C: Context> super::SvmPointer<$atomic> for $svm<C> {
                type Context = C;

                #[inline(always)]
                fn allocator (&self) -> &Svm<C> {
                    SvmBox::allocator(&self.0)
                }

                #[inline(always)]
                fn as_ptr (&self) -> *const $atomic {
                    <SvmBox<[$ty], C> as super::SvmPointer::<$ty>>::as_ptr(&self.0).cast()
                }

                #[inline(always)]
                fn as_mut_ptr (&mut self) -> *mut $atomic {
                    <SvmBox<[$ty], C> as super::SvmPointer::<$ty>>::as_mut_ptr(&mut self.0).cast()
                }

                #[inline(always)]
                fn len (&self) -> usize {
                    <[$ty]>::len(&self.0)
                }
            }

            #[cfg(target_has_atomic = $len)]
            unsafe impl<C: Context> KernelPointer<$atomic> for $svm<C> where C: 'static + Send + Clone {
                #[inline(always)]
                unsafe fn set_arg (&self, kernel: &mut RawKernel, _wait: &mut WaitList, idx: u32) -> Result<()> {
                    kernel.set_svm_argument::<$atomic, Self>(idx, self)?;
                    // SVM atomic pointers are allways fine grained
                    Ok(())
                }

                #[inline(always)]
                fn complete (&self, _event: &RawEvent) -> Result<()> {
                    // SVM atomic pointers are allways fine grained
                    Ok(())
                }
            }
        )+
    };
}

impl_atomic! {
    "32" in i32 => AtomicU32 as SvmAtomicI32,
    "32" in u32 => AtomicU32 as SvmAtomicU32,
    "64" in i64 => AtomicI64 as SvmAtomicI64,
    "64" in u64 => AtomicU64 as SvmAtomicU64,
    "ptr" in isize => AtomicIsize as SvmAtomicIsize,
    "ptr" in usize => AtomicUsize as SvmAtomicUsize
}