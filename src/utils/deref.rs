use std::ops::{Deref, DerefMut};

#[repr(transparent)]
pub struct DerefWrapper<T> (T);

impl<T> DerefWrapper<T> {
    #[inline(always)]
    pub const fn new (v: T) -> Self {
        Self (v)
    }

    #[inline(always)]
    pub fn into_inner (self) -> T {
        self.0
    }
}

impl<T> Deref for DerefWrapper<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DerefWrapper<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}