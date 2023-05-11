use std::{
    alloc::{Layout, LayoutError},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[repr(C)]
union PtrRepr<T: ?Sized> {
    const_ptr: *const T,
    mut_ptr: *mut T,
    components: PtrComponents,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct PtrComponents {
    data_address: *const (),
    metadata: *const (),
}

pub struct ThinFn<F: ?Sized + FnClosure> {
    inner: NonNull<u8>,
    _phtm: PhantomData<F>,
}

impl<T: ?Sized + FnClosure> ThinFn<T> {
    pub fn new<F: IntoFnClosure<T>>(f: F) -> Self
    where
        F: FnClosureNew,
    {
        let (layout, f_offset) = calculate_layout::<F>().expect("unexpected layout error");
        let Some(ptr) = NonNull::new(unsafe { std::alloc::alloc(layout) }) else { std::alloc::handle_alloc_error(layout) };

        unsafe {
            let raw = ptr.as_ptr().add(f_offset);
            raw.sub(core::mem::size_of::<*const ()>())
                .cast::<*const ()>()
                .write(f.metadata());
            raw.cast::<F>().write(f);

            return Self {
                inner: NonNull::new_unchecked(raw).cast(),
                _phtm: PhantomData,
            };
        }
    }

    #[inline]
    pub unsafe fn from_raw(ptr: *mut ()) -> Self {
        return Self {
            inner: NonNull::new_unchecked(ptr.cast()),
            _phtm: PhantomData,
        };
    }

    #[inline]
    pub fn into_raw(self) -> *mut () {
        let this = ManuallyDrop::new(self);
        return this.inner.as_ptr().cast();
    }

    #[inline]
    pub fn metadata(&self) -> *const () {
        unsafe {
            self.inner
                .as_ptr()
                .sub(core::mem::size_of::<*const ()>())
                .cast::<*const ()>()
                .read()
        }
    }

    pub fn as_ptr(&self) -> *const T {
        unsafe {
            PtrRepr {
                components: PtrComponents {
                    data_address: self.inner.as_ptr().cast(),
                    metadata: self.metadata(),
                },
            }
            .const_ptr
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe {
            PtrRepr {
                components: PtrComponents {
                    data_address: self.inner.as_ptr().cast(),
                    metadata: self.metadata(),
                },
            }
            .mut_ptr
        }
    }
}

impl<F: ?Sized + FnClosure> Deref for ThinFn<F> {
    type Target = F;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<F: ?Sized + FnClosure> DerefMut for ThinFn<F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<F: ?Sized + FnClosure> Drop for ThinFn<F> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            struct DeallocGuard(*mut u8, Layout);
            impl Drop for DeallocGuard {
                #[inline]
                fn drop(&mut self) {
                    let (layout, offset) =
                        unsafe { Layout::new::<*const ()>().extend(self.1).unwrap_unchecked() };

                    unsafe {
                        std::alloc::dealloc(self.0.sub(offset), layout);
                    }
                }
            }

            let guard = DeallocGuard(self.inner.as_ptr(), Layout::for_value(self.deref()));
            core::ptr::drop_in_place(self.as_mut_ptr());
            drop(guard);
        }
    }
}

unsafe impl<F: ?Sized + FnClosure + Send> Send for ThinFn<F> {}
unsafe impl<F: ?Sized + FnClosure + Sync> Sync for ThinFn<F> {}

#[doc(hidden)]
pub unsafe trait FnClosure: sealed::Sealed {}
#[doc(hidden)]
pub trait FnClosureNew: FnClosure {}

#[doc(hidden)]
pub unsafe trait IntoFnClosure<F: ?Sized + FnClosure> {
    unsafe fn metadata(&self) -> *const ();
}

macro_rules! impl_fn_closure {
    (
        ($($arg:ident),*): $($trait:ident),*
    ) => {
        impl<'a, $($arg,)* __T__> sealed::Sealed for dyn 'a + $($trait+)* FnOnce($($arg),*) -> __T__ {}
        impl<'a, $($arg,)* __T__> sealed::Sealed for dyn 'a + $($trait+)* Fn($($arg),*) -> __T__ {}
        impl<'a, $($arg,)* __T__> sealed::Sealed for dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__ {}

        impl<'a, $($arg,)* __T__> FnClosureNew for dyn 'a + $($trait+)* Fn($($arg),*) -> __T__ {}
        impl<'a, $($arg,)* __T__> FnClosureNew for dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__ {}

        unsafe impl<'a, $($arg,)* __T__> FnClosure for dyn 'a + $($trait+)* Fn($($arg),*) -> __T__ {}
        unsafe impl<'a, $($arg,)* __T__, __F__: 'a + $($trait+)* Fn($($arg),*) -> __T__> IntoFnClosure<dyn 'a + $($trait+)* Fn($($arg),*) -> __T__> for __F__ {
            unsafe fn metadata(&self) -> *const () {
                PtrRepr {
                    const_ptr: self as *const (dyn 'a + $($trait+)* Fn($($arg),*) -> __T__)
                }.components.metadata
            }
        }

        unsafe impl<'a, $($arg,)* __T__> FnClosure for dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__ {}
        unsafe impl<'a, $($arg,)* __T__, __F__: 'a + $($trait+)* FnMut($($arg),*) -> __T__> IntoFnClosure<dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__> for __F__ {
            unsafe fn metadata(&self) -> *const () {
                PtrRepr {
                    const_ptr: self as *const (dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__)
                }.components.metadata
            }
        }

        unsafe impl<'a, $($arg,)* __T__> FnClosure for dyn 'a + $($trait+)* FnOnce($($arg),*) -> __T__ {}
        unsafe impl<'a, $($arg,)* __T__, __F__: 'a + $($trait+)* FnOnce($($arg),*) -> __T__> IntoFnClosure<dyn 'a + $($trait+)* FnOnce($($arg),*) -> __T__> for __F__ {
            unsafe fn metadata(&self) -> *const () {
                PtrRepr {
                    const_ptr: self as *const (dyn 'a + $($trait+)* FnOnce($($arg),*) -> __T__)
                }.components.metadata
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg,)* __T__> ThinFn<dyn 'a + $($trait+)* FnOnce($($arg),*) -> __T__> {
            pub fn new_once<__F__: 'a + $($trait+)* FnOnce($($arg),*) -> __T__>(f: __F__) -> Self
            {
                #[inline(always)]
                fn cast_ptr_to<T, F> (ptr: *mut T, f: &F) -> *mut F {
                    ptr.cast::<F>()
                }

                let mut f = ManuallyDrop::new(f);
                let f = move |$($arg),*| unsafe {
                    (ManuallyDrop::take(&mut f))($($arg),*)
                };

                let metadata = unsafe {
                    PtrRepr {
                        const_ptr: &f as *const (dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__)
                    }.components.metadata
                };

                let (layout, f_offset) = calculate_layout_of(&f).expect("unexpected layout error");
                let Some(ptr) = NonNull::new(unsafe { std::alloc::alloc(layout) }) else { std::alloc::handle_alloc_error(layout) };

                unsafe {
                    let raw = ptr.as_ptr().add(f_offset);
                    raw.sub(core::mem::size_of::<*const ()>())
                        .cast::<*const ()>()
                        .write(metadata);
                    cast_ptr_to(raw, &f).write(f);

                    return Self {
                        inner: NonNull::new_unchecked(raw).cast(),
                        _phtm: PhantomData,
                    };
                }
            }

            #[inline]
            pub fn call_once (self, ($($arg,)*): ($($arg,)*)) -> __T__ {
                // execute as fnmut
                todo!()
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg,)* __T__> ThinFn<dyn 'a + $($trait+)* Fn($($arg),*) -> __T__> {
            #[inline]
            pub fn call (&self, ($($arg,)*): ($($arg,)*)) -> __T__ {
                (self.deref())($($arg),*)
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($arg,)* __T__> ThinFn<dyn 'a + $($trait+)* FnMut($($arg),*) -> __T__> {
            #[inline]
            pub fn call_mut (&mut self, ($($arg,)*): ($($arg,)*)) -> __T__ {
                (self.deref_mut())($($arg),*)
            }
        }
    };

    (
        $(
            ($($arg:ident),*)
        ),+
    ) => {
        $(
            impl_fn_closure! {
                ($($arg),*):
            }

            impl_fn_closure! {
                ($($arg),*): Send
            }

            impl_fn_closure! {
                ($($arg),*): Sync
            }

            impl_fn_closure! {
                ($($arg),*): Send, Sync
            }
        )+
    };
}

impl_fn_closure! {
    (),
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)
}

fn calculate_layout<F>() -> Result<(Layout, usize), LayoutError> {
    let layout = Layout::new::<*const ()>(); // metadata
    let (layout, f) = layout.extend(Layout::new::<F>())?;
    return Ok((layout.pad_to_align(), f));
}

fn calculate_layout_of<F>(f: &F) -> Result<(Layout, usize), LayoutError> {
    let layout = Layout::new::<*const ()>(); // metadata
    let (layout, f) = layout.extend(Layout::for_value(f))?;
    return Ok((layout.pad_to_align(), f));
}

mod sealed {
    pub trait Sealed {}
}
