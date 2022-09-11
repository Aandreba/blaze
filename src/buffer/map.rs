use std::{marker::PhantomData, ops::{Deref, DerefMut}, ffi::c_void};
use crate::{prelude::{Context, Global}, memobj::MapPtr, event::consumer::Consumer};
use super::Buffer;

pub struct Map<'scope, 'env: 'scope, T: Copy, C: Context> {
    ptr: *const c_void,
    phtm: PhantomData<&'env Buffer<T, C>>,
    scope: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope, 'env, T: Copy, C: Context> Consumer<'scope, MapGuard<'env, T, C>> for Map<'scope, 'env, T, C> {
    fn consume (self) -> crate::prelude::Result<MapGuard<'env, T, C>> {
        todo!()
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

/// Guard for a read-write map of a [`Buffer`]
pub struct MutMapGuard<'a, T: Copy, C: Context = Global> {
    ptr: MapPtr<T, C>,
    phtm: PhantomData<&'a mut Buffer<T, C>>
}

impl<'a, T: Copy, C: Context> Deref for MutMapGuard<'a, T, C> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.ptr.ptr
        }
    }
}

impl<'a, T: Copy, C: Context> DerefMut for MutMapGuard<'a, T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ptr.ptr
        }
    }
}