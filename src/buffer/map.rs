use std::{marker::PhantomData, ops::{Deref, DerefMut}, ffi::c_void, fmt::Debug};
use crate::{prelude::{Context, Global, Event}, memobj::MapPtr, event::consumer::Consumer};
use super::Buffer;

pub type BufferMapEvent<'scope, 'env, T, C> = Event<BufferMap<'scope, 'env, T, C>>;
pub type BufferMapMutEvent<'scope, 'env, T, C> = Event<BufferMapMut<'scope, 'env, T, C>>;

pub struct BufferMap<'scope, 'env: 'scope, T: Copy, C: Context> {
    ptr: *const c_void,
    buff: &'env Buffer<T, C>,
    len: usize,
    scope: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope, 'env, T: Copy, C: Context> BufferMap<'scope, 'env, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: *const c_void, buff: &'env Buffer<T, C>, len: usize) -> Self {
        Self {
            ptr, buff, len,
            scope: PhantomData
        }
    }
}

impl<'scope, 'env, T: Copy, C: Context> Consumer<'scope> for BufferMap<'scope, 'env, T, C> where C: Clone {
    type Output = MapGuard<'env, T, C>;
    
    #[inline]
    fn consume (self) -> crate::prelude::Result<MapGuard<'env, T, C>> {
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(self.ptr as *mut T, self.len)
        };
        let ptr = MapPtr::new(ptr, self.buff.inner.clone().into(), self.buff.ctx.clone());
        return Ok(MapGuard::new(ptr))
    }
}

pub struct BufferMapMut<'scope, 'env: 'scope, T: Copy, C: Context> {
    ptr: *const c_void,
    buff: &'env mut Buffer<T, C>,
    len: usize,
    scope: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope, 'env, T: Copy, C: Context> BufferMapMut<'scope, 'env, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: *const c_void, buff: &'env mut Buffer<T, C>, len: usize) -> Self {
        Self {
            ptr, buff, len,
            scope: PhantomData
        }
    }
}

impl<'scope, 'env, T: Copy, C: Context> Consumer<'scope> for BufferMapMut<'scope, 'env, T, C> where C: Clone {
    type Output = MapMutGuard<'env, T, C>;

    #[inline]
    fn consume (self) -> crate::prelude::Result<Self::Output> {
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(self.ptr as *mut T, self.len)
        };
        let ptr = MapPtr::new(ptr, self.buff.inner.clone().into(), self.buff.ctx.clone());
        return Ok(MapMutGuard::new(ptr))
    }
}

/// Guard for a read-only map of a [`Buffer`]
pub struct MapGuard<'a, T: Copy, C: Context = Global> {
    ptr: MapPtr<T, C>,
    phtm: PhantomData<&'a Buffer<T, C>>
}

impl<'a, T: Copy, C: Context> MapGuard<'a, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: MapPtr<T, C>) -> Self {
        Self { ptr, phtm: PhantomData }
    }
}

impl<'a, T: Copy, C: Context> Deref for MapGuard<'a, T, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr.ptr
        }
    }
}

impl<'a, T: Debug + Copy, C: Context> Debug for MapGuard<'_, T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

/// Guard for a read-write map of a [`Buffer`]
pub struct MapMutGuard<'a, T: Copy, C: Context = Global> {
    ptr: MapPtr<T, C>,
    phtm: PhantomData<&'a mut Buffer<T, C>>
}

impl<'a, T: Copy, C: Context> MapMutGuard<'a, T, C> {
    #[inline(always)]
    pub(super) fn new (ptr: MapPtr<T, C>) -> Self {
        Self { ptr, phtm: PhantomData }
    }
}

impl<'a, T: Copy, C: Context> Deref for MapMutGuard<'a, T, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr.ptr
        }
    }
}

impl<'a, T: Copy, C: Context> DerefMut for MapMutGuard<'a, T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ptr.ptr
        }
    }
}

impl<'a, T: Debug + Copy, C: Context> Debug for MapMutGuard<'_, T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}