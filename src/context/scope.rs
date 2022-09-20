use std::{sync::{Arc, atomic::{AtomicUsize, Ordering, AtomicI32}}, marker::PhantomData, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}};
use opencl_sys::CL_SUCCESS;
use crate::{prelude::{Result, RawCommandQueue, RawEvent, Event, Error}, event::consumer::{Consumer, Noop, NoopEvent}};
use super::{Global, Context};
use blaze_proc::docfg;

#[derive(Clone)]
enum ScopeWaker {
    Thread (std::thread::Thread),
    #[cfg(feature = "futures")]
    Flag (Arc<::futures::task::AtomicWaker>)
}

impl ScopeWaker {
    #[inline(always)]
    pub fn wake (&self) {
        match self {
            Self::Thread(x) => x.unpark(),
            #[cfg(feature = "futures")]
            Self::Flag(x) => x.wake()
        }
    }
}

/// A scope to enqueue events in.\
/// See [`scope`] and [`local_scope`]
pub struct Scope<'scope, 'env: 'scope, C: 'env + Context = Global> {
    ctx: &'env C,
    data: Arc<(AtomicUsize, AtomicI32)>,
    thread: ScopeWaker,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>
}

impl<'scope, 'env: 'scope, C: 'env + Context> Scope<'scope, 'env, C> {
    #[cfg(feature = "futures")]
    #[doc(hidden)]
    #[inline(always)]
    pub fn new_async (ctx: &'env C) -> Self {
        return Scope {
            ctx,
            data: Arc::new((AtomicUsize::new(0), AtomicI32::new(CL_SUCCESS))),
            thread: ScopeWaker::Flag(Arc::new(futures::task::AtomicWaker::new())),
            scope: PhantomData,
            env: PhantomData
        }
    }

    #[cfg(feature = "futures")]
    #[doc(hidden)]
    #[inline]
    pub async unsafe fn wait_async (&self) {
        use std::task::Poll;

        if let ScopeWaker::Flag(ref flag) = self.thread {
            return futures::future::poll_fn(move |cx| {
                flag.register(cx.waker());
                if self.data.0.load(::std::sync::atomic::Ordering::Acquire) == 0 {
                    return Poll::Ready(())
                }
                return Poll::Pending
            }).await;
        }

        std::hint::unreachable_unchecked()
    }

    #[cfg(feature = "futures")]
    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn get_data (&self) -> &(AtomicUsize, AtomicI32) {
        &self.data
    }
 
    /// Enqueues a new event within the scope.
    pub fn enqueue<E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>, F: Consumer<'scope>> (&'scope self, supplier: E, consumer: F) -> Result<Event<F>> {
        let queue = self.ctx.next_queue();
        let inner = supplier(&queue)?;
        let evt = Event::new(inner, consumer);

        if self.data.0.fetch_add(1, Ordering::Relaxed) == usize::MAX {
            panic!("too many events in scope")
        }

        let queue_size = queue.size.clone();
        let scope_data = self.data.clone();
        let scope_thread = self.thread.clone();

        evt.on_complete(move |_, res| {
            drop(queue_size);

            if let Err(e) = res {
                let _ = scope_data.1.compare_exchange(CL_SUCCESS, e.ty.as_i32(), Ordering::Relaxed, Ordering::Relaxed);
            }

            if scope_data.0.fetch_sub(1, Ordering::Relaxed) == 1 {
                scope_thread.wake();
            }
        }).unwrap();

        return Ok(evt)
    }

    /// Enqueues a new [`NoopEvent`] within the scope.
    #[inline(always)]
    pub fn enqueue_noop<E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>> (&'scope self, supplier: E) -> Result<NoopEvent<'scope>> {
        self.enqueue(supplier, Noop::new())
    }
}

/// Creates a new scope with the global context to enqueue events in.
/// All events that haven't completed by the end of the function will be automatically awaitad before the function returns.
#[inline(always)]
pub fn scope<'env, T, F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> Result<T>> (f: F) -> Result<T> {
    local_scope(Global::get(), f)
}

/// Creates a new scope with the specified context to enqueue events in.
/// All events that haven't completed by the end of the function will be automatically joined before the function returns.
pub fn local_scope<'env, T, C: 'env + Context, F: for<'scope> FnOnce(&'scope Scope<'scope, 'env, C>) -> Result<T>> (ctx: &'env C, f: F) -> Result<T> {
    let scope = Scope {
        ctx,
        data: Arc::new((AtomicUsize::new(0), AtomicI32::new(CL_SUCCESS))),
        thread: ScopeWaker::Thread(std::thread::current()),
        scope: PhantomData,
        env: PhantomData
    };

    // Run `f`, but catch panics so we can make sure to wait for all the threads to join.
    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));
    
    // Wait until all the events are finished.
    while scope.data.0.load(Ordering::Acquire) != 0 {
        std::thread::park();
    }

    // Throw any panic from `f`, or the return value of `f`.
    return match result {
        Err(e) => resume_unwind(e),
        Ok(x) => {
            let e = scope.data.1.load(Ordering::Relaxed);
            if e != CL_SUCCESS {
                return Err(Error::from(e));
            }
            x
        }
    }
}

/// Creates a new scope with the specified context to enqueue events in.
/// All events that haven't completed by the end of the future will be automatically awaited before the future returns.
///
/// # Example
/// ```rust
/// let ctx = SimpleContext::default()?;
/// let mut buffer = Buffer::new_in(ctx.clone(), &[1, 2, 3, 4, 5], MemAccess::default(), false)?;

/// let v = local_scope_async!(
///     &ctx,
///     |s| async {
///         let v = buffer.read(s, .., None)?.join_async()?.await?;
///         println!("{v:?}");
///         let _ = buffer.read(s, .., None)?;
///         Ok(())
///     }
/// ).await;
/// 
/// buffer.write_blocking(1, &[8, 9], None)?;
/// ```
#[docfg(feature = "futures")]
#[macro_export]
macro_rules! local_scope_async {
    ($ctx:expr, |$s:ident| $exp:expr) => {
        async {
            #[doc(hidden)]
            #[inline(always)]
            fn __catch_unwind__<'scope, 'env: 'scope, T, C: 'env + $crate::prelude::Context, Fut: 'scope + ::std::future::Future<Output = $crate::prelude::Result<T>>, F: ::std::ops::FnOnce(&'scope $crate::prelude::Scope<'scope, 'env, C>) -> Fut> (s: &'scope $crate::prelude::Scope<'scope, 'env, C>, f: F) -> ::std::result::Result<Fut, ::std::boxed::Box<dyn ::std::any::Any + ::std::marker::Send>> {
                return ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| f(s)))
            }

            let __scope__ = $crate::prelude::Scope::new_async($ctx);

            // Run `f`, but catch panics so we can make sure to wait for all the threads to join.
            let __result__ = match __catch_unwind__(&__scope__, |$s| $exp) {
                Ok(x) => $crate::futures::FutureExt::catch_unwind(::std::panic::AssertUnwindSafe(x)).await,
                Err(e) => Err(e)
            };

            // Wait until all the events are finished.
            unsafe {
                $crate::prelude::Scope::wait_async(&__scope__).await;
            }

            // Throw any panic from `f`, or the return value of `f`.
            match __result__ {
                Err(e) => ::std::panic::resume_unwind(e),
                Ok(x) => unsafe {
                    let e = __scope__.get_data().1.load(::std::sync::atomic::Ordering::Relaxed);
                    if e == 0 {
                        x
                    } else {
                        Err($crate::prelude::Error::from(e))
                    }
                }
            }
        }
    };
}

/// Creates a new scope with the global context to enqueue events in.
/// All events that haven't completed by the end of the future will be automatically awaited before the future returns.
///
/// # Example
/// ```rust
/// let mut buffer = Buffer::new_in(ctx.clone(), &[1, 2, 3, 4, 5], MemAccess::default(), false)?;

/// let v = scope_async!(
///     |s| async {
///         let v = buffer.read(s, .., None)?.join_async()?.await?;
///         println!("{v:?}");
///         let _ = buffer.read(s, .., None)?;
///         Ok(())
///     }
/// ).await;
/// 
/// buffer.write_blocking(1, &[8, 9], None)?;
/// ```
#[docfg(feature = "futures")]
#[macro_export]
macro_rules! scope_async {
    (|$s:ident| $exp:expr) => {
        $crate::local_scope_async! {
            $crate::prelude::Global::get(),
            |$s| $exp
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "futures")]
    #[tokio::test]
    async fn test () -> crate::prelude::Result<()> {
        use crate::prelude::*;

        let ctx = SimpleContext::default()?;
        let mut buffer = Buffer::new_in(ctx.clone(), &[1, 2, 3, 4, 5], MemAccess::default(), false)?;

        let _ = local_scope_async!(
            &ctx,
            |s| async {
                let v = buffer.read(s, .., None)?.join_async()?.await?;
                println!("{v:?}");
                Ok(())
            }
        ).await;

        buffer.write_blocking(1, &[8, 9], None)?;

        return Ok(())
    }
}

/*
```rust
#[global_context]
static CTX : SimpleContext = SimpleContext::default();

fn manually () {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false);

    scope(|s| {
        let _ = buffer.write(s, 2, &[6], &[]);
    });

    scope(|s| {
        let v = buffer.read(s, .., &[])?.join()?;
        assert_eq!(v.as_slice(), &[1, 2, 6, 4, 5]);
    })
}

#[scoped(s)] // it may default to `s`
fn auto_v1 () {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false);
    
    scope(|s| {
        let _ = buffer.write(s, 2, &[6], &[]);
        todo!();
    });

    let v = buffer.read(s, .., &[])?.join()?;
    assert_eq!(v.as_slice(), &[1, 2, 6, 4, 5]);
}
```
*/