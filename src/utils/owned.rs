use std::{sync::Arc, marker::PhantomData, ops::{Deref, DerefMut}};
use parking_lot::{lock_api::{Mutex, RawMutex}};

pub trait OwnedMutex {
    type Target: ?Sized;
    type Raw: RawMutex;
    
    fn lock_owned (self: Arc<Self>) -> OwnedMutexGuard<Self::Raw, Self::Target>;
}

impl<R: RawMutex, T: ?Sized> OwnedMutex for Mutex<R, T> {
    type Target = T;
    type Raw = R;

    #[inline(always)]
    fn lock_owned (self: Arc<Self>) -> OwnedMutexGuard<Self::Raw, Self::Target> {
        unsafe {
            self.raw().lock();
        }

        OwnedMutexGuard { mutex: self, marker: PhantomData }
    }
}

/// Owned mutex guard
pub struct OwnedMutexGuard<R: RawMutex, T: ?Sized> {
    mutex: Arc<Mutex<R, T>>,
    marker: PhantomData<T>
}

impl<R: RawMutex, T: ?Sized> Deref for OwnedMutexGuard<R, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data_ptr() }
    }
}

impl<R: RawMutex, T: ?Sized> DerefMut for OwnedMutexGuard<R, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data_ptr() }
    }
}

impl<R: RawMutex, T: ?Sized> Drop for OwnedMutexGuard<R, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.mutex.raw().unlock()
        }
    }
}