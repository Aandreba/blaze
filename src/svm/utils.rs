use std::{collections::VecDeque, ops::{Deref, DerefMut}};
use crate::{context::{Global, Context}};
use super::{Svm};

/// Object that wraps, in some way, a pointer to SVM memory
/// # Safety
/// For an [`SvmPointer`] implementation to be safe, [`as_ptr`](SvmPointer::as_ptr) and [`as_mut_ptr`](SvmPointer::as_mut_ptr) must return the same SVM-allocated pointer.
pub unsafe trait SvmPointer<T: ?Sized> {
    type Context: Context;

    /// Returns a reference to the underlying [`Svm`] allocator
    fn allocator (&self) -> &Svm<Self::Context>;
    /// Returns the SVM pointer
    fn as_ptr (&self) -> *const T;
    /// Returns the mutable SVM pointer
    fn as_mut_ptr (&mut self) -> *mut T; 
    /// Returns the number of elements owned by the pointer
    fn len (&self) -> usize;
}

/// A [`Box`] with an [`Svm`] allocator
pub type SvmBox<T, C = Global> = Box<T, Svm<C>>;
/// A [`Vec`] with an [`Svm`] allocator
pub type SvmVec<T, C = Global> = Vec<T, Svm<C>>;
/// A [`VecDeque`] with an [`Svm`] allocator
pub type SvmVecDeque<T, C = Global> = VecDeque<T, Svm<C>>;

unsafe impl<T: ?Sized, C: Context> SvmPointer<T> for SvmBox<T, C> {
    type Context = C;

    #[inline(always)]
    fn allocator (&self) -> &Svm<C> {
        Box::allocator(self)
    }

    #[inline(always)]
    fn as_ptr (&self) -> *const T {
        self.deref()
    }

    #[inline(always)]
    fn as_mut_ptr (&mut self) -> *mut T {
        self.deref_mut()
    }

    #[inline(always)]
    fn len (&self) -> usize {
        1
    }
}

unsafe impl<T, C: Context> SvmPointer<T> for SvmBox<[T], C> {
    type Context = C;

    #[inline(always)]
    fn allocator (&self) -> &Svm<C> {
        Box::allocator(self)
    }

    #[inline(always)]
    fn as_ptr (&self) -> *const T {
        <[T]>::as_ptr(self)
    }

    #[inline(always)]
    fn as_mut_ptr (&mut self) -> *mut T {
        <[T]>::as_mut_ptr(self)
    }

    #[inline(always)]
    fn len (&self) -> usize {
        <[T]>::len(self)
    }
}

unsafe impl<T, C: Context> SvmPointer<T> for SvmVec<T, C> {
    type Context = C;

    #[inline(always)]
    fn allocator (&self) -> &Svm<C> {
        Vec::allocator(self)
    }

    #[inline(always)]
    fn as_ptr (&self) -> *const T {
        Vec::as_ptr(self)
    }

    #[inline(always)]
    fn as_mut_ptr (&mut self) -> *mut T {
        Vec::as_mut_ptr(self)
    }

    #[inline(always)]
    fn len (&self) -> usize {
        Vec::len(self)
    }
}

unsafe impl<T, C: Context> SvmPointer<[T]> for SvmVec<T, C> {
    type Context = C;

    #[inline(always)]
    fn allocator (&self) -> &Svm<C> {
        Vec::allocator(self)
    }

    #[inline(always)]
    fn as_ptr (&self) -> *const [T] {
        unsafe {
            core::slice::from_raw_parts(Vec::as_ptr(self), self.len())
        }
    }

    #[inline(always)]
    fn as_mut_ptr (&mut self) -> *mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(Vec::as_mut_ptr(self), self.len())
        }
    }

    #[inline(always)]
    fn len (&self) -> usize {
        1
    }
}