use std::ops::{Deref, DerefMut};

#[repr(transparent)]
pub struct DerefCell<T>(pub T);

impl<T> Deref for DerefCell<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DerefCell<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}