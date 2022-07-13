use std::ops::{Deref, DerefMut};
use super::MemObject;

/// Represents types that directly or indirectly dereference [`MemObject`]
pub trait AsMem {
    fn as_mem (&self) -> &MemObject;
}

impl AsMem for MemObject {
    #[inline(always)]
    fn as_mem (&self) -> &MemObject {
        self
    }
}

impl<T: Deref<Target = impl 'static + AsMem>> AsMem for T {
    #[inline]
    fn as_mem (&self) -> &MemObject {
        self.deref().as_mem()
    }
}

/// Represents types that directly or indirectly mutably dereference [`MemObject`]
pub trait AsMutMem: AsMem {
    fn as_mut_mem (&mut self) -> &mut MemObject;
}

impl AsMutMem for MemObject {
    #[inline(always)]
    fn as_mut_mem (&mut self) -> &mut MemObject {
        self
    }
}

impl<T: DerefMut<Target = impl 'static + AsMutMem>> AsMutMem for T {
    #[inline]
    fn as_mut_mem (&mut self) -> &mut MemObject {
        self.deref_mut().as_mut_mem()
    }
}