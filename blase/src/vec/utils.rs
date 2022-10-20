use std::{mem::{MaybeUninit, transmute}, marker::PhantomData, cmp::Ordering};
use blaze_rs::{event::{Consumer}, prelude::{Buffer, Context, Global}};
use super::EucVec;

pub struct Binary<'a, T: Copy> {
    inner: EucVec<MaybeUninit<T>>,
    phtm: PhantomData<(&'a EucVec<T>, &'a EucVec<T>)>
}

impl<'a, T: Copy> Binary<'a, T> {
    #[inline(always)]
    pub(super) const fn new (inner: EucVec<MaybeUninit<T>>) -> Self {
        Self { inner, phtm: PhantomData }
    }
}

impl<T: Copy> Consumer for Binary<'_, T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn consume (self) -> blaze_rs::prelude::Result<Self::Output> {
        unsafe { Ok(self.inner.assume_init()) }
    }
}

impl<T: Copy> IncompleteConsumer for Binary<'_, T> {
    type Incomplete = EucVec<MaybeUninit<T>>;

    #[inline(always)]
    fn consume_incomplete (self) -> blaze_rs::prelude::Result<Self::Incomplete> {
        return Ok(self.inner)
    }
}

/// Wrapper arround [`num_traits::real::Real::sqrt`] method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Sqrt<T> (PhantomData<T>);

impl<T> Sqrt<T> {
    #[inline(always)]
    pub const fn new () -> Self { Self(PhantomData) }
}

impl<T: num_traits::real::Real> FnOnce<(T,)> for Sqrt<T> {
    type Output = T;

    #[inline(always)]
    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output {
        T::sqrt(x)
    }
}

/// Wrapper arround [`core::mem::transmute`] from `Vec<i8>` to `Vec<Ordering>`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct TransmuteTotalOrdering;

impl FnOnce<(Vec<i8>,)> for TransmuteTotalOrdering {
    type Output = Vec<Ordering>;

    #[inline(always)]
    extern "rust-call" fn call_once(self, (x,): (Vec<i8>,)) -> Self::Output {
        unsafe { transmute(x) }
    }
}

/// Wrapper arround [`core::mem::transmute`] from `Vec<i8>` to `Vec<Option<Ordering>>`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct TransmuteOrdering;

impl FnOnce<(Vec<i8>,)> for TransmuteOrdering {
    type Output = Vec<Option<Ordering>>;

    #[inline(always)]
    extern "rust-call" fn call_once(self, (x,): (Vec<i8>,)) -> Self::Output {
        unsafe { transmute(x) }
    }
}

/// Wrapper arround [`assume_init`](blaze_rs::buffer::Buffer::assume_init)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct AssumeInit<T, C: Context = Global> (PhantomData<(T, C)>);

impl<T, C: Context> AssumeInit<T, C> {
    #[inline(always)]
    pub const unsafe fn new () -> Self { Self(PhantomData) }
}

impl<T: Copy, C: Context> FnOnce<(Buffer<MaybeUninit<T>, C>,)> for AssumeInit<T, C> {
    type Output = Buffer<T, C>;

    #[inline(always)]
    extern "rust-call" fn call_once(self, (x,): (Buffer<MaybeUninit<T>, C>,)) -> Self::Output {
        unsafe { x.assume_init() }
    }
}