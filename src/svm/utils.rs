use std::{collections::VecDeque, ops::{Deref, DerefMut}};
use crate::{context::{Global, Context}};
use super::{Svm};
use sealed::Sealed;

pub(super) mod sealed {
    pub trait Sealed {}
}

/// Object that wraps, in some way, a pointer to SVM memory
pub unsafe trait SvmPointer {
    type Type: ?Sized;
    type Context: Context;

    /// Returns a reference to the underlying [`Svm`] allocator
    fn allocator (&self) -> &Svm<Self::Context>;
    /// Returns the SVM pointer
    fn as_ptr (&self) -> *const Self::Type;
    /// Returns the mutable SVM pointer
    fn as_mut_ptr (&mut self) -> *mut Self::Type; 
    /// Returns the number of elements owned by the pointer
    fn len (&self) -> usize;
}

/// A [`Box`] with an [`Svm`] allocator
pub type SvmBox<T, C = Global> = Box<T, Svm<C>>;
/// A [`Vec`] with an [`Svm`] allocator
pub type SvmVec<T, C = Global> = Vec<T, Svm<C>>;
/// A [`VecDeque`] with an [`Svm`] allocator
pub type SvmVecDeque<T, C = Global> = VecDeque<T, Svm<C>>;

impl<T: ?Sized, C: Context> Sealed for SvmBox<T, C> {}
impl<T, C: Context> Sealed for SvmVec<T, C> {}
impl<T, C: Context> Sealed for SvmVecDeque<T, C> {}

unsafe impl<T: ?Sized, C: Context> SvmPointer for SvmBox<T, C> {
    type Type = T;
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

unsafe impl<T, C: Context> SvmPointer for SvmVec<T, C> {
    type Type = T;
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