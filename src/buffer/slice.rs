use std::{marker::PhantomData, ops::{Deref, DerefMut}, fmt::Debug};
use crate::prelude::*;
use super::IntoRange;

/// An immutable slice of a [`Buffer`]
#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[repr(transparent)]
pub struct Buf<'a, T, C: Context = Global> {
    inner: Buffer<T, C>,
    phtm: PhantomData<&'a Buffer<T, C>>
}

impl<'a, T, C: Context> Buf<'a, T, C> {
    #[inline]
    pub fn new<R: IntoRange> (parent: &'a Buffer<T, C>, range: R) -> Result<Self> where C: Clone {
        let region = range.into_range::<T>(parent)?;
        let inner = unsafe {
            parent.create_sub_buffer(MemAccess::READ_ONLY, region)?
        };

        return Ok(Self { 
            inner: Buffer { inner, ctx: parent.ctx.clone(), phtm: PhantomData }, 
            phtm: PhantomData
        })
    } 
}

impl<'a, T, C: Context> Deref for Buf<'a, T, C> {
    type Target = Buffer<T, C>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Debug, C: Context> Debug for Buf<'_, T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: PartialEq, C: Context> PartialEq for Buf<'_, T, C> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(other)
    }
}

impl<T: Eq, C: Context> Eq for Buf<'_, T, C> {}

/// A mutable slice of a [`Buffer`]
#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[repr(transparent)]
pub struct BufMut<'a, T, C: Context = Global> {
    inner: Buffer<T, C>,
    phtm: PhantomData<&'a mut Buffer<T, C>>
}

impl<'a, T, C: Context> BufMut<'a, T, C> {
    #[inline]
    pub fn new<R: IntoRange> (parent: &'a mut Buffer<T, C>, range: R) -> Result<Self> where C: Clone {
        let region = range.into_range::<T>(parent)?;
        let inner = unsafe {
            parent.create_sub_buffer(MemAccess::READ_WRITE, region)?
        };

        return Ok(Self { 
            inner: Buffer { inner, ctx: parent.ctx.clone(), phtm: PhantomData }, 
            phtm: PhantomData
        })
    } 
}

impl<'a, T, C: Context> Deref for BufMut<'a, T, C> {
    type Target = Buffer<T, C>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T, C: Context> DerefMut for BufMut<'a, T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Debug, C: Context> Debug for BufMut<'_, T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: PartialEq, C: Context> PartialEq for BufMut<'_, T, C> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(other)
    }
}

impl<T: Eq, C: Context> Eq for BufMut<'_, T, C> {}