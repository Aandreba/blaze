use std::{collections::VecDeque, ops::{Deref, DerefMut}, mem::ManuallyDrop, ffi::c_void};
use crate::{core::*, context::{Global, Context}, event::{WaitList, RawEvent}};
use super::{Svm};
use sealed::Sealed;

pub(super) mod sealed {
    pub trait Sealed {}
}

const ALLOC : Svm = Svm::new();

/// Object that wraps, in some way, a pointer to SVM memory
pub unsafe trait SvmPointer<C: Context = Global> {
    type Type: ?Sized;

    /// Returns a reference to the underlying [`Svm`] allocator
    fn allocator (&self) -> &Svm<C>;
    /// Returns the SVM pointer
    fn as_ptr (&self) -> *const Self::Type;
    /// Returns the mutable SVM pointer
    fn as_mut_ptr (&mut self) -> *mut Self::Type; 
    /// Returns the number of elements owned by the pointer
    fn len (&self) -> usize;

    /// Drops the pointer after the events in the [`WaitList`] have completed
    #[inline]
    unsafe fn drop_after (self, wait: impl Into<WaitList>) -> Result<RawEvent> where Self: Sized {
        let mut this = ManuallyDrop::new(self);
        let ptr = this.as_mut_ptr();
        let alloc = this.allocator();
        
        match alloc.enqueue_free(&[ptr.cast()], wait) {
            Ok(x) => Ok(x),
            Err(e) => {
                ManuallyDrop::drop(&mut this);
                Err(e)
            }
        }
    } 
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

unsafe impl<T: ?Sized, C: Context> SvmPointer<C> for SvmBox<T, C> {
    type Type = T;

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

unsafe impl<T, C: Context> SvmPointer<C> for SvmVec<T, C> {
    type Type = T;

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

// BOX
pub trait SvmBoxExt<T: ?Sized>: Sealed {
    fn new (v: T) -> Self where T: Sized;
}

impl<T: ?Sized> SvmBoxExt<T> for SvmBox<T> {
    #[inline(always)]
    fn new (v: T) -> Self where T: Sized {
        Self::new_in(v, ALLOC)
    }
} 

// VEC
pub trait SvmVecExt<T>: Sealed + Sized {
    fn with_capacity (cap: usize) -> Self;

    #[inline(always)]
    fn new () -> Self {
        Self::with_capacity(0)
    }
}

impl<T> SvmVecExt<T> for SvmVec<T> {
    #[inline(always)]
    fn with_capacity (cap: usize) -> Self {
        Self::with_capacity_in(cap, ALLOC)
    }
}

// VEC DEQUEUE
pub trait SvmVecDequeExt<T>: Sealed + Sized {
    fn with_capacity (cap: usize) -> Self;

    #[inline(always)]
    fn new () -> Self {
        const INITIAL_CAPACITY: usize = 7; // 2^3 - 1
        Self::with_capacity(INITIAL_CAPACITY)
    }
}

impl<T> SvmVecDequeExt<T> for SvmVecDeque<T> {
    #[inline(always)]
    fn with_capacity (cap: usize) -> Self {
        Self::with_capacity_in(cap, ALLOC)
    }
}