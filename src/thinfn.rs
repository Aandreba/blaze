use std::{
    alloc::{Layout, LayoutError},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
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

pub struct ThinFn<F: ?Sized> {
    inner: NonNull<()>,
    _phtm: PhantomData<F>,
}

impl<T: FnClosure> ThinFn<T> {
    pub fn new<F: IntoFnClosure<T>>(f: F) -> Self {
        let (layout, f_offset) = calculate_layout::<T>().expect("unexpected layout error");
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout)
        }

        unsafe {
            ptr.cast::<*const ()>().write(f.metadata());
            ptr.add(f_offset).cast::<F>().write(f);

            return Self {
                inner: NonNull::new(ptr.cast()),
                _phtm: PhantomData,
            };
        }
    }

    #[inline]
    pub unsafe fn from_raw(ptr: *mut ()) -> Self {
        return Self {
            inner: NonNull::new(ptr).expect("nu"),
            _phtm: PhantomData,
        };
    }

    #[inline]
    pub fn into_raw(self) -> *mut () {
        let this = ManuallyDrop::new(self);
        return self.inner.as_ptr();
    }

    #[inline]
    fn metadata(&self) -> *const () {}
}

impl<F: ?Sized> Deref for ThinFn<F> {
    type Target = F;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {}
    }
}

impl<F: ?Sized> Drop for ThinFn<F> {
    #[inline]
    fn drop(&mut self) {
        todo!()
    }
}

unsafe trait FnClosure {}
unsafe trait IntoFnClosure<F: ?Sized + FnClosure> {
    unsafe fn metadata(&self) -> *const ();
}

macro_rules! impl_fn_closure {
    (
        $(
            ($($arg:ident),*)
        ),+
    ) => {
        $(
            unsafe impl<'a, __T__> FnClosure for dyn 'a + Fn($($arg),*) -> __T__ {}
            unsafe impl<'a, __T__, __F__: 'a + Fn($($arg),*) -> __T__> IntoFnClosure<dyn 'a + Fn($($arg),*) -> __T__> for __F__ {
                unsafe fn metadata(&self) -> *const () {
                    PtrRepr {
                        const_ptr: self as *const (dyn 'a + Fn($($arg),*) -> __T__)
                    }.components.metadata
                }
            }
        )+
    };
}

impl_fn_closure! {
    ()
}

fn calculate_layout<F>() -> Result<(Layout, usize), LayoutError> {
    let layout = Layout::new::<*const ()>(); // metadata
    let (layout, f) = layout.extend(Layout::new::<F>())?;
    return Ok((layout.pad_to_align(), f));
}
