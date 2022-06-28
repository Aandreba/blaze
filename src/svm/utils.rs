use std::{collections::VecDeque, ops::{Deref, DerefMut}};
use crate::context::{Global, Context};
use super::{Svm};
use sealed::Sealed;

pub(super) mod sealed {
    pub trait Sealed {}
}

const ALLOC : Svm = Svm::new();

pub trait SvmPointer: Sealed {
    type Type: ?Sized;

    fn as_ptr (&self) -> *const Self::Type;
    fn as_mut_ptr (&mut self) -> *mut Self::Type; 
    fn len (&self) -> usize;
}

pub type SvmBox<T, C = Global> = Box<T, Svm<C>>;
pub type SvmVec<T, C = Global> = Vec<T, Svm<C>>;
pub type SvmVecDeque<T, C = Global> = VecDeque<T, Svm<C>>;

impl<T: ?Sized, C: Context> Sealed for SvmBox<T, C> {}
impl<T, C: Context> Sealed for SvmVec<T, C> {}
impl<T, C: Context> Sealed for SvmVecDeque<T, C> {}

impl<T: ?Sized, C: Context> SvmPointer for SvmBox<T, C> {
    type Type = T;

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

impl<T, C: Context> SvmPointer for SvmVec<T, C> {
    type Type = T;

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