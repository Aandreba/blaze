use std::mem::MaybeUninit;
use blaze_proc::blaze;

/*#[blaze(Arith)]
#[link = "hello"]
pub extern "C" {
    #[cfg(debug_assertions)]
    #[link_name = "hello"]
    fn test (n: u32, lhs: *const f32, rhs: *const f32, out: *mut MaybeUninit<f32>);
}*/

pub struct Arith<C: ::blaze_rs::context::Context = ::blaze_rs::context::Global> {
    #[doc(hidden)]
    __blaze_inner__: ::blaze_rs::core::RawProgram,
    #[doc(hidden)]
    __blaze_ctx__: C,
    test: ::std::sync::Mutex<::blaze_rs::core::RawKernel>,
}
impl Arith {
    #[inline(always)]
    pub fn new<'a>(
        options: impl Into<Option<&'a str>>,
    ) -> ::blaze_rs::core::Result<Self> {
        Self::new_in(::blaze_rs::context::Global, options)
    }
}
impl<C: ::blaze_rs::context::Context> Arith<C> {
    pub fn new_in<'a>(
        ctx: C,
        options: impl Into<Option<&'a str>>,
    ) -> ::blaze_rs::core::Result<Self> {
        let __blaze_ctx__ = ctx;
        let (__blaze_inner__, __blaze_kernels__) = ::blaze_rs::core::RawProgram::from_source_in(
            &__blaze_ctx__,
            "hello",
            options,
        )?;
        let mut test = None;
        for __blaze_kernel__ in __blaze_kernels__.into_iter() {
            match __blaze_kernel__.name()?.as_str() {
                "test" => test = unsafe { Some(__blaze_kernel__.clone()) },
                _ => {}
            }
        }
        let test = match test {
            Some(__x) => ::std::sync::Mutex::new(__x),
            None => {
                return Err(
                    ::blaze_rs::core::Error::new(
                        ::blaze_rs::core::ErrorKind::InvalidKernel,
                        "kernel \'test\' not found",
                    ),
                );
            }
        };
        Ok(Self {
            __blaze_inner__,
            __blaze_ctx__,
            test,
        })
    }
}
impl<C: ::blaze_rs::context::Context> ::std::ops::Deref for Arith<C> {
    type Target = ::blaze_rs::core::RawProgram;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.__blaze_inner__
    }
}
impl<C: ::blaze_rs::context::Context> Arith<C> {
    pub unsafe fn test<
        '__scope__,
        '__env__,
        const N: usize,
        LHS: ::blaze_rs::buffer::KernelPointer<f32>,
        RHS: ::blaze_rs::buffer::KernelPointer<f32>,
        OUT: ::blaze_rs::buffer::KernelPointer<MaybeUninit<f32>>,
    >(
        &self,
        scope: &'__scope__ ::blaze_rs::context::Scope<'__scope__, '__env__, C>,
        n: u32,
        lhs: &'__env__ LHS,
        rhs: &'__env__ RHS,
        out: &'__env__ mut OUT,
        global_work_dims: [usize; N],
        local_work_dims: impl Into<Option<[usize; N]>>,
        wait: &[::blaze_rs::event::RawEvent],
    ) -> ::blaze_rs::prelude::Result<::blaze_rs::event::NoopEvent<'__scope__>> {
        let mut wait = wait.to_vec();
        let mut __blaze_kernel__ = match self.test.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        __blaze_kernel__.set_argument(0u32, n)?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            lhs,
            &mut __blaze_kernel__,
            &mut wait,
            1u32,
        )?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            rhs,
            &mut __blaze_kernel__,
            &mut wait,
            2u32,
        )?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            out,
            &mut __blaze_kernel__,
            &mut wait,
            3u32,
        )?;
        let __blaze_inner__ = __blaze_kernel__
            .enqueue_with_scope(&scope, global_work_dims, local_work_dims, &wait)?;
        drop(__blaze_kernel__);
        ::blaze_rs::buffer::KernelPointer::complete(lhs, &__blaze_inner__)?;
        ::blaze_rs::buffer::KernelPointer::complete(rhs, &__blaze_inner__)?;
        ::blaze_rs::buffer::KernelPointer::complete(out, &__blaze_inner__)?;
        return Ok(__blaze_inner__);
    }
    pub unsafe fn test_blocking<
        const N: usize,
        LHS: ::blaze_rs::buffer::KernelPointer<f32>,
        RHS: ::blaze_rs::buffer::KernelPointer<f32>,
        OUT: ::blaze_rs::buffer::KernelPointer<MaybeUninit<f32>>,
    >(
        &self,
        n: u32,
        lhs: &LHS,
        rhs: &RHS,
        out: &mut OUT,
        global_work_dims: [usize; N],
        local_work_dims: impl Into<Option<[usize; N]>>,
        wait: &[::blaze_rs::event::RawEvent],
    ) -> ::blaze_rs::prelude::Result<()> {
        let mut wait = wait.to_vec();
        let mut __blaze_kernel__ = match self.test.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };
        __blaze_kernel__.set_argument(0u32, n)?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            lhs,
            &mut __blaze_kernel__,
            &mut wait,
            1u32,
        )?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            rhs,
            &mut __blaze_kernel__,
            &mut wait,
            2u32,
        )?;
        ::blaze_rs::buffer::KernelPointer::set_arg(
            out,
            &mut __blaze_kernel__,
            &mut wait,
            3u32,
        )?;
        let __blaze_inner__ = unsafe {
            __blaze_kernel__
                .enqueue_unchecked(
                    ::blaze_rs::context::Context::next_queue(&self.__blaze_ctx__),
                    global_work_dims,
                    local_work_dims,
                    &wait,
                )?
        };
        drop(__blaze_kernel__);
        ::blaze_rs::buffer::KernelPointer::complete(lhs, &__blaze_inner__)?;
        ::blaze_rs::buffer::KernelPointer::complete(rhs, &__blaze_inner__)?;
        ::blaze_rs::buffer::KernelPointer::complete(out, &__blaze_inner__)?;
        return __blaze_inner__.join_by_ref();
    }
}