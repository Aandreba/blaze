use std::{marker::PhantomData, ops::{Deref, DerefMut}, ffi::c_void, fmt::Debug};
use crate::{prelude::{Context, Global, Event}, memobj::MapPtr, event::consumer::Consumer};
use super::Buffer;

pub type BufferMapEvent<'scope, 'env, T, C = Global> = Event<BufferMap<'scope, 'env, T, C>>;
pub type BufferMapMutEvent<'scope, 'env, T, C = Global> = Event<BufferMapMut<'scope, 'env, T, C>>;

pub struct BufferMap<'scope, 'env: 'scope, T, C: Context> {
    ptr: *const c_void,
    buff: &'env Buffer<T, C>,
    len: usize,
    scope: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope, 'env, T, C: Context> BufferMap<'scope, 'env, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: *const c_void, buff: &'env Buffer<T, C>, len: usize) -> Self {
        Self {
            ptr, buff, len,
            scope: PhantomData
        }
    }
}

impl<'scope, 'env, T, C: Context> Consumer for BufferMap<'scope, 'env, T, C> {
    type Output = MapGuard<'env, T, C>;
    
    #[inline]
    fn consume (self) -> crate::prelude::Result<MapGuard<'env, T, C>> {
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(self.ptr as *mut T, self.len)
        };
        let ptr = MapPtr::new(ptr, self.buff.inner.clone().into(), &self.buff.ctx);
        return Ok(MapGuard::new(ptr))
    }
}

pub struct BufferMapMut<'scope, 'env: 'scope, T, C: Context> {
    ptr: *const c_void,
    buff: &'env mut Buffer<T, C>,
    len: usize,
    scope: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope, 'env, T, C: Context> BufferMapMut<'scope, 'env, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: *const c_void, buff: &'env mut Buffer<T, C>, len: usize) -> Self {
        Self {
            ptr, buff, len,
            scope: PhantomData
        }
    }
}

impl<'scope, 'env, T, C: Context> Consumer for BufferMapMut<'scope, 'env, T, C> {
    type Output = MapMutGuard<'env, T, C>;

    #[inline]
    fn consume (self) -> crate::prelude::Result<Self::Output> {
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(self.ptr as *mut T, self.len)
        };
        let ptr = MapPtr::new(ptr, self.buff.inner.clone().into(), &self.buff.ctx);
        return Ok(MapMutGuard::new(ptr))
    }
}

/// Guard for a read-only map of a [`Buffer`]
pub struct MapGuard<'a, T, C: Context = Global> {
    ptr: MapPtr<T, &'a C>,
    phtm: PhantomData<&'a Buffer<T, C>>
}

impl<'a, T: 'a, C: Context> IntoIterator for MapGuard<'a, T, C> {
    type Item = &'a T;
    type IntoIter = MapIter<'a, T, C>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MapIter::new(self)
    }
}

impl<'a, T, C: Context> MapGuard<'a, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: MapPtr<T, &'a C>) -> Self {
        Self { ptr, phtm: PhantomData }
    }
}

impl<'a, T, C: Context> Deref for MapGuard<'a, T, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr.ptr
        }
    }
}

impl<'a, T: Debug, C: Context> Debug for MapGuard<'_, T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

/// Guard for a read-write map of a [`Buffer`]
pub struct MapMutGuard<'a, T, C: Context = Global> {
    ptr: MapPtr<T, &'a C>,
    phtm: PhantomData<&'a mut Buffer<T, C>>
}

impl<'a, T, C: Context> MapMutGuard<'a, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: MapPtr<T, &'a C>) -> Self {
        Self { ptr, phtm: PhantomData }
    }

    /// Converts a [`MapMutGuard`] into a [`MapGuard`].
    #[inline(always)]
    pub fn into_read (self) -> MapGuard<'a, T, C> {
        MapGuard { ptr: self.ptr, phtm: PhantomData }
    }
}

impl<'a, T: 'a, C: Context> IntoIterator for MapMutGuard<'a, T, C> {
    type Item = &'a mut T;
    type IntoIter = MapMutIter<'a, T, C>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MapMutIter::new(self)
    }
}

impl<'a, T, C: Context> Deref for MapMutGuard<'a, T, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr.ptr
        }
    }
}

impl<'a, T, C: Context> DerefMut for MapMutGuard<'a, T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ptr.ptr
        }
    }
}

impl<'a, T: Debug, C: Context> Debug for MapMutGuard<'_, T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

pub struct MapIter<'a, T, C: Context = Global> {
    #[allow(unused)]
    guard: MapGuard<'a, T, C>,
    inner: std::slice::Iter<'a, T>
}

impl<'a, T, C: Context> MapIter<'a, T, C> {
    #[inline(always)]
    fn new (guard: MapGuard<'a, T, C>) -> Self {
        let inner = unsafe {
            (&mut *guard.ptr.ptr).iter()
        };
        Self { guard, inner }
    }
}

impl<'a, T: 'a, C: Context> Iterator for MapIter<'a, T, C> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct MapMutIter<'a, T, C: Context = Global> {
    #[allow(unused)]
    guard: MapMutGuard<'a, T, C>,
    inner: std::slice::IterMut<'a, T>
}

impl<'a, T, C: Context> MapMutIter<'a, T, C> {
    #[inline(always)]
    fn new (guard: MapMutGuard<'a, T, C>) -> Self {
        let inner = unsafe {
            (&mut *guard.ptr.ptr).iter_mut()
        };
        Self { guard, inner }
    }
}

impl<'a, T: 'a, C: Context> Iterator for MapMutIter<'a, T, C> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}