use std::ops::{Deref, DerefMut};
use super::RawMemObject;

/// Represents types that directly or indirectly dereference [`RawMemObject`](crate::prelude::RawMemObject)
pub trait AsMem {
    fn as_mem (&self) -> &RawMemObject;
}

impl AsMem for RawMemObject {
    #[inline(always)]
    fn as_mem (&self) -> &RawMemObject {
        self
    }
}

impl<T: Deref<Target = impl 'static + AsMem>> AsMem for T {
    #[inline]
    fn as_mem (&self) -> &RawMemObject {
        self.deref().as_mem()
    }
}

/// Represents types that directly or indirectly mutably dereference [`RawMemObject`](crate::prelude::RawMemObject)
pub trait AsMutMem: AsMem {
    fn as_mut_mem (&mut self) -> &mut RawMemObject;
}

impl AsMutMem for RawMemObject {
    #[inline(always)]
    fn as_mut_mem (&mut self) -> &mut RawMemObject {
        self
    }
}

impl<T: DerefMut<Target = impl 'static + AsMutMem>> AsMutMem for T {
    #[inline]
    fn as_mut_mem (&mut self) -> &mut RawMemObject {
        self.deref_mut().as_mut_mem()
    }
}