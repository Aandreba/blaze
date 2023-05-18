use blaze_rs::{
    buffer,
    prelude::{blaze, global_context, Buffer, MemAccess, Result, SimpleContext},
};
use std::mem::MaybeUninit;

#[global_context]
static CONTEXT: SimpleContext = SimpleContext::default();

/*
#[blaze(pub FloatTanh)]
#[link = KERNEL]
extern "C" {
    fn forward(n: u64, x_buffer: *mut f32);
    fn backward(n: u64, x_buffer: *const f32, y_buffer: *mut MaybeUninit<f32>);
}
*/

/* MANUALLY */
const KERNEL: &str = r#"
    __kernel void forward (ulong n, __global float* x_buffer) {
        for (ulong i = get_global_id(0); i < n; i += get_global_size(0)) {
            x_buffer[i] = tanh(x_buffer[i]);
        }
    }

    __kernel void backward (ulong n, const __global float* x_buffer, __global float* y_buffer) {
        for (ulong i = get_global_id(0); i < n; i += get_global_size(0)) {
            const float c = cosh(x_buffer[i]);
            y_buffer[i] = 1.0 / (c * c);
        }
    }
    "#;
pub struct FloatTanh<C: ::blaze_rs::context::Context = ::blaze_rs::context::Global> {
    #[doc(hidden)]
    __blaze_inner__: ::blaze_rs::core::RawProgram,
    #[doc(hidden)]
    __blaze_ctx__: C,
    forward: ::std::sync::Mutex<::blaze_rs::core::RawKernel>,
    backward: ::std::sync::Mutex<::blaze_rs::core::RawKernel>,
}
impl FloatTanh {
    #[inline(always)]
    pub fn new(options: Option<&str>) -> ::blaze_rs::core::Result<Self> {
        Self::new_in(::blaze_rs::context::Global, options)
    }
}
impl<C: ::blaze_rs::context::Context> FloatTanh<C> {
    pub fn new_in(ctx: C, options: Option<&str>) -> ::blaze_rs::core::Result<Self> {
        let __blaze_ctx__ = ctx;
        let (__blaze_inner__, __blaze_kernels__) =
            ::blaze_rs::core::RawProgram::from_source_in(&__blaze_ctx__, KERNEL, options)?;
        #[allow(unused_doc_comments)]
        let mut forward = None;
        #[allow(unused_doc_comments)]
        let mut backward = None;
        for __blaze_kernel__ in __blaze_kernels__.into_iter() {
            match __blaze_kernel__.name()?.as_str() {
                #[allow(unused_doc_comments)]
                "forward" => forward = unsafe { Some(__blaze_kernel__.clone()) },
                #[allow(unused_doc_comments)]
                "backward" => backward = unsafe { Some(__blaze_kernel__.clone()) },
                _ => {}
            }
        }
        #[allow(unused_doc_comments)]
        let forward = match forward {
            Some(__x) => ::std::sync::Mutex::new(__x),
            None => {
                return Err(::blaze_rs::core::Error::new(
                    ::blaze_rs::core::ErrorKind::InvalidKernel,
                    "kernel \'forward\' not found",
                ));
            }
        };
        #[allow(unused_doc_comments)]
        let backward = match backward {
            Some(__x) => ::std::sync::Mutex::new(__x),
            None => {
                return Err(::blaze_rs::core::Error::new(
                    ::blaze_rs::core::ErrorKind::InvalidKernel,
                    "kernel \'backward\' not found",
                ));
            }
        };
        Ok(Self {
            __blaze_inner__,
            __blaze_ctx__,
            #[allow(unused_doc_comments)]
            forward,
            #[allow(unused_doc_comments)]
            backward,
        })
    }
    /// Returns the context of the program
    #[inline]
    pub fn context(&self) -> &C {
        &self.__blaze_ctx__
    }
}
impl<C: ::blaze_rs::context::Context> ::std::ops::Deref for FloatTanh<C> {
    type Target = ::blaze_rs::core::RawProgram;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.__blaze_inner__
    }
}
struct Forward<'__scope__, X_BUFFER>(::core::marker::PhantomData<(u64, &'__scope__ mut X_BUFFER)>);
impl<'__scope__, X_BUFFER> blaze_rs::event::Consumer for Forward<'__scope__, X_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ mut X_BUFFER)>: blaze_rs::event::Consumer,
{
    type Output = <::core::marker::PhantomData<
        (u64, &'__scope__ mut X_BUFFER),
    > as blaze_rs::event::Consumer>::Output;
    #[inline(always)]
    unsafe fn consume(self) -> blaze_rs::prelude::Result<Self::Output> {
        <::core::marker::PhantomData<
            (u64, &'__scope__ mut X_BUFFER),
        > as blaze_rs::event::Consumer>::consume(self.0)
    }
}
impl<'__scope__, X_BUFFER> ::core::fmt::Debug for Forward<'__scope__, X_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ mut X_BUFFER)>: ::core::fmt::Debug,
{
    #[inline(always)]
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(&self.0, f)
    }
}
impl<'__scope__, X_BUFFER> ::core::clone::Clone for Forward<'__scope__, X_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ mut X_BUFFER)>: ::core::clone::Clone,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(::core::clone::Clone::clone(&self.0))
    }
}
impl<'__scope__, X_BUFFER> ::core::marker::Copy for Forward<'__scope__, X_BUFFER> where
    ::core::marker::PhantomData<(u64, &'__scope__ mut X_BUFFER)>: ::core::marker::Copy
{
}
type ForwardEvent<'__scope__, X_BUFFER> = ::blaze_rs::event::Event<Forward<'__scope__, X_BUFFER>>;
impl<C: ::blaze_rs::context::Context> FloatTanh<C> {
    unsafe fn forward<
        '__scope__,
        '__env__: '__scope__,
        X_BUFFER: ::blaze_rs::buffer::KernelPointer<f32>,
        const N: usize,
    >(
        &self,
        scope: &'__scope__ ::blaze_rs::context::Scope<'__scope__, '__env__, C>,
        n: u64,
        x_buffer: &'__env__ mut X_BUFFER,
        global_work_dims: [usize; N],
        local_work_dims: impl Into<Option<[usize; N]>>,
        wait: ::blaze_rs::WaitList,
    ) -> ::blaze_rs::prelude::Result<ForwardEvent<'__scope__, X_BUFFER>> {
        let mut wait = match wait {
            ::blaze_rs::WaitList::Some(x) => x.to_vec(),
            ::blaze_rs::WaitList::None => ::std::vec::Vec::new(),
        };
        let mut __blaze_kernel__ = match self.forward.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        __blaze_kernel__.set_argument(0u32, n)?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            x_buffer,
            &mut __blaze_kernel__,
            &mut wait,
            1u32,
        )?;
        let __blaze_inner__ = __blaze_kernel__.enqueue_phantom_with_scope(
            &scope,
            global_work_dims,
            local_work_dims,
            Some(&wait),
        )?;
        drop(__blaze_kernel__);
        let __blaze_inner__ = ::blaze_rs::event::Event::map_consumer(__blaze_inner__, Forward);
        ::blaze_rs::buffer::KernelPointer::complete(x_buffer, &__blaze_inner__)?;
        return Ok(__blaze_inner__);
    }
    unsafe fn forward_blocking<const N: usize, X_BUFFER: ::blaze_rs::buffer::KernelPointer<f32>>(
        &self,
        n: u64,
        x_buffer: &mut X_BUFFER,
        global_work_dims: [usize; N],
        local_work_dims: impl Into<Option<[usize; N]>>,
        wait: ::blaze_rs::WaitList,
    ) -> ::blaze_rs::prelude::Result<()> {
        let mut wait = match wait {
            ::blaze_rs::WaitList::Some(x) => x.to_vec(),
            ::blaze_rs::WaitList::None => ::std::vec::Vec::new(),
        };
        let mut __blaze_kernel__ = match self.forward.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        __blaze_kernel__.set_argument(0u32, n)?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            x_buffer,
            &mut __blaze_kernel__,
            &mut wait,
            1u32,
        )?;
        let __blaze_inner__ = unsafe {
            __blaze_kernel__.enqueue_unchecked(
                ::blaze_rs::context::Context::next_queue(&self.__blaze_ctx__),
                global_work_dims,
                local_work_dims,
                Some(&wait),
            )?
        };
        drop(__blaze_kernel__);
        ::blaze_rs::buffer::KernelPointer::complete(x_buffer, &__blaze_inner__)?;
        return __blaze_inner__.join_by_ref();
    }
}
struct Backward<'__scope__, X_BUFFER, Y_BUFFER>(
    ::core::marker::PhantomData<(u64, &'__scope__ X_BUFFER, &'__scope__ mut Y_BUFFER)>,
);
impl<'__scope__, X_BUFFER, Y_BUFFER> blaze_rs::event::Consumer
    for Backward<'__scope__, X_BUFFER, Y_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ X_BUFFER, &'__scope__ mut Y_BUFFER)>:
        blaze_rs::event::Consumer,
{
    type Output = <::core::marker::PhantomData<(
        u64,
        &'__scope__ X_BUFFER,
        &'__scope__ mut Y_BUFFER,
    )> as blaze_rs::event::Consumer>::Output;
    #[inline(always)]
    unsafe fn consume(self) -> blaze_rs::prelude::Result<Self::Output> {
        <::core::marker::PhantomData<
            (u64, &'__scope__ X_BUFFER, &'__scope__ mut Y_BUFFER),
        > as blaze_rs::event::Consumer>::consume(self.0)
    }
}
impl<'__scope__, X_BUFFER, Y_BUFFER> ::core::fmt::Debug for Backward<'__scope__, X_BUFFER, Y_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ X_BUFFER, &'__scope__ mut Y_BUFFER)>:
        ::core::fmt::Debug,
{
    #[inline(always)]
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(&self.0, f)
    }
}
impl<'__scope__, X_BUFFER, Y_BUFFER> ::core::clone::Clone
    for Backward<'__scope__, X_BUFFER, Y_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ X_BUFFER, &'__scope__ mut Y_BUFFER)>:
        ::core::clone::Clone,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(::core::clone::Clone::clone(&self.0))
    }
}
impl<'__scope__, X_BUFFER, Y_BUFFER> ::core::marker::Copy
    for Backward<'__scope__, X_BUFFER, Y_BUFFER>
where
    ::core::marker::PhantomData<(u64, &'__scope__ X_BUFFER, &'__scope__ mut Y_BUFFER)>:
        ::core::marker::Copy,
{
}
type BackwardEvent<'__scope__, X_BUFFER, Y_BUFFER> =
    ::blaze_rs::event::Event<Backward<'__scope__, X_BUFFER, Y_BUFFER>>;
impl<C: ::blaze_rs::context::Context> FloatTanh<C> {
    unsafe fn backward<
        '__scope__,
        '__env__: '__scope__,
        X_BUFFER: ::blaze_rs::buffer::KernelPointer<f32>,
        Y_BUFFER: ::blaze_rs::buffer::KernelPointer<MaybeUninit<f32>>,
        const N: usize,
    >(
        &self,
        scope: &'__scope__ ::blaze_rs::context::Scope<'__scope__, '__env__, C>,
        n: u64,
        x_buffer: &'__env__ X_BUFFER,
        y_buffer: &'__env__ mut Y_BUFFER,
        global_work_dims: [usize; N],
        local_work_dims: impl Into<Option<[usize; N]>>,
        wait: ::blaze_rs::WaitList,
    ) -> ::blaze_rs::prelude::Result<BackwardEvent<'__scope__, X_BUFFER, Y_BUFFER>> {
        let mut wait = match wait {
            ::blaze_rs::WaitList::Some(x) => x.to_vec(),
            ::blaze_rs::WaitList::None => ::std::vec::Vec::new(),
        };
        let mut __blaze_kernel__ = match self.backward.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        __blaze_kernel__.set_argument(0u32, n)?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            x_buffer,
            &mut __blaze_kernel__,
            &mut wait,
            1u32,
        )?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            y_buffer,
            &mut __blaze_kernel__,
            &mut wait,
            2u32,
        )?;
        let __blaze_inner__ = __blaze_kernel__.enqueue_phantom_with_scope(
            &scope,
            global_work_dims,
            local_work_dims,
            Some(&wait),
        )?;
        drop(__blaze_kernel__);
        let __blaze_inner__ = ::blaze_rs::event::Event::map_consumer(__blaze_inner__, Backward);
        ::blaze_rs::buffer::KernelPointer::complete(x_buffer, &__blaze_inner__)?;
        ::blaze_rs::buffer::KernelPointer::complete(y_buffer, &__blaze_inner__)?;
        return Ok(__blaze_inner__);
    }
    unsafe fn backward_blocking<
        const N: usize,
        X_BUFFER: ::blaze_rs::buffer::KernelPointer<f32>,
        Y_BUFFER: ::blaze_rs::buffer::KernelPointer<MaybeUninit<f32>>,
    >(
        &self,
        n: u64,
        x_buffer: &X_BUFFER,
        y_buffer: &mut Y_BUFFER,
        global_work_dims: [usize; N],
        local_work_dims: impl Into<Option<[usize; N]>>,
        wait: ::blaze_rs::WaitList,
    ) -> ::blaze_rs::prelude::Result<()> {
        let mut wait = match wait {
            ::blaze_rs::WaitList::Some(x) => x.to_vec(),
            ::blaze_rs::WaitList::None => ::std::vec::Vec::new(),
        };
        let mut __blaze_kernel__ = match self.backward.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        __blaze_kernel__.set_argument(0u32, n)?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            x_buffer,
            &mut __blaze_kernel__,
            &mut wait,
            1u32,
        )?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            y_buffer,
            &mut __blaze_kernel__,
            &mut wait,
            2u32,
        )?;
        let __blaze_inner__ = unsafe {
            __blaze_kernel__.enqueue_unchecked(
                ::blaze_rs::context::Context::next_queue(&self.__blaze_ctx__),
                global_work_dims,
                local_work_dims,
                Some(&wait),
            )?
        };
        drop(__blaze_kernel__);
        ::blaze_rs::buffer::KernelPointer::complete(x_buffer, &__blaze_inner__)?;
        ::blaze_rs::buffer::KernelPointer::complete(y_buffer, &__blaze_inner__)?;
        return __blaze_inner__.join_by_ref();
    }
}

#[test]
fn gemm() -> Result<()> {
    const M: usize = 5;
    const N: usize = 3;
    const K: usize = 2;

    let tanh = FloatTanh::new(None)?;
    std::hint::black_box(tanh);

    Ok(())
}
