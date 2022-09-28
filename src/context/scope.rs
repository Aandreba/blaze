use std::{sync::{Arc, atomic::{AtomicUsize, Ordering, AtomicI32}}, marker::{PhantomData}, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}, thread::Thread};
use opencl_sys::CL_SUCCESS;
use crate::{prelude::{Result, RawCommandQueue, RawEvent, Event, Error}, event::consumer::{Consumer, Noop, NoopEvent, PhantomEvent}};
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
    /// Creates a new [`Event`] scope in the current thread
    #[inline(always)]
    pub fn new (ctx: &'env C) -> Self {
        Self::with_thread(ctx, std::thread::current())
    }

    /// Creates a new [`Event`] scope targeted to `async` use
    #[docfg(feature = "futures")]
    #[inline(always)]
    pub fn new_async (ctx: &'env C) -> Self {
        Self::with_waker(ctx, Arc::new(futures::task::AtomicWaker::new()))
    }
    
    /// Creates a new [`Event`] scope with the specified thread
    #[inline(always)]
    pub fn with_thread (ctx: &'env C, thread: Thread) -> Self {
        Self {
            ctx,
            data: Arc::new((AtomicUsize::new(0), AtomicI32::new(CL_SUCCESS))),
            thread: ScopeWaker::Thread(thread),
            scope: PhantomData,
            env: PhantomData
        }
    }

    /// Creates a new `async` [`Event`] scope with the specified waker 
    #[docfg(feature = "futures")]
    #[inline(always)]
    pub fn with_waker (ctx: &'env C, waker: Arc<futures::task::AtomicWaker>) -> Self {
        Self {
            ctx,
            data: Arc::new((AtomicUsize::new(0), AtomicI32::new(CL_SUCCESS))),
            thread: ScopeWaker::Flag(waker),
            scope: PhantomData,
            env: PhantomData
        }
    }

    /// Enqueues a new event within the scope.
    pub fn enqueue<E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>, F: 'scope + Consumer> (&'scope self, supplier: E, consumer: F) -> Result<Event<F>> {
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
    pub fn enqueue_noop<E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>> (&'scope self, supplier: E) -> Result<NoopEvent> {
        self.enqueue(supplier, Noop)
    }

    /// Enqueues a new [`NoopEvent`] within the scope.
    #[inline(always)]
    pub fn enqueue_phantom<T: 'scope, E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>> (&'scope self, supplier: E) -> Result<PhantomEvent<T>> {
        self.enqueue(supplier, PhantomData)
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

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        use futures::Future;
        use std::task::Poll;

        enum AsyncScopeFuture<T, Fut> {
            Future (futures::future::CatchUnwind<AssertUnwindSafe<Fut>>),
            Panic (Box<dyn std::any::Any + Send>),
            Value (Result<T>),
            Ended
        }

        #[doc(hidden)]
        pub struct InnerAsyncScope<'scope, 'env: 'scope, T, Fut: 'scope + Future<Output = Result<T>>, C: 'env + Context> {
            scope: &'scope Scope<'scope, 'env, C>,
            fut: AsyncScopeFuture<T, Fut>,
            _pin: std::marker::PhantomPinned
        }

        impl<'scope, 'env: 'scope, T, Fut: 'scope + Future<Output = Result<T>>, C: 'env + Context> InnerAsyncScope<'scope, 'env, T, Fut, C> {
            pub unsafe fn new<F: FnOnce(&'scope Scope<'scope, 'env, C>) -> Fut> (scope: &'scope Scope<'scope, 'env, C>, f: F) -> Self {
                let fut = match catch_unwind(AssertUnwindSafe(|| f(scope))) {
                    Ok(f) => AsyncScopeFuture::Future(futures::FutureExt::catch_unwind(AssertUnwindSafe(f))),
                    Err(e) => AsyncScopeFuture::Panic(e)
                };

                return Self { scope, fut, _pin: std::marker::PhantomPinned };
            }

            #[inline(always)]
            fn get_waker (&self) -> &futures::task::AtomicWaker {
                match self.scope.thread {
                    ScopeWaker::Flag(ref x) => x,
                    _ => unsafe { std::hint::unreachable_unchecked() }
                }
            }
        }

        impl<'scope, 'env, T, Fut: 'scope + Future<Output = Result<T>>, C: 'env + Context> Future for InnerAsyncScope<'scope, 'env, T, Fut, C> {
            type Output = Result<T>;

            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                let this = unsafe {
                    self.get_unchecked_mut()
                };
                
                // Wait future
                if let AsyncScopeFuture::Future(ref mut fut) = this.fut {
                    // Safety: Self is `!Unpin` and has already been pinned, so it cannot move
                    match unsafe { std::pin::Pin::new_unchecked(fut).poll(cx) } {
                        Poll::Ready(Ok(x)) => this.fut = AsyncScopeFuture::Value(x),
                        Poll::Ready(Err(e)) => this.fut = AsyncScopeFuture::Panic(e),
                        Poll::Pending => return Poll::Pending
                    }
                }
                
                // Sleep
                this.get_waker().register(cx.waker());
                if this.scope.data.0.load(Ordering::Acquire) != 0 {
                    return std::task::Poll::Pending;
                }

                // Complete
                match core::mem::replace(&mut this.fut, AsyncScopeFuture::Ended) {
                    AsyncScopeFuture::Panic(e) => resume_unwind(e),
                    AsyncScopeFuture::Value(x) => {
                        let e = this.scope.data.1.load(Ordering::Relaxed);
                        if e == CL_SUCCESS {
                            return std::task::Poll::Ready(x)
                        }
                        return std::task::Poll::Ready(Err(Error::from(e)));
                    },
                    AsyncScopeFuture::Ended => panic!("Scope already finished"),
                    #[cfg(debug_assertions)]
                    AsyncScopeFuture::Future(_) => unreachable!(),
                    #[cfg(not(debug_assertions))]
                    AsyncScopeFuture::Future(_) => unreachable_unchecked()
                }
            }
        }

        impl<'scope, 'env, T, Fut: Future<Output = Result<T>>, C: 'env + Context> Drop for InnerAsyncScope<'scope, 'env, T, Fut, C> {
            #[inline]
            fn drop(&mut self) {
                // Await already-started events, without starting new ones.
                let thread = unsafe {
                    std::mem::transmute::<_, *const ()>(std::thread::current())
                };

                let waker = std::task::RawWaker::new(thread, &TABLE);
                let waker = unsafe { std::task::Waker::from_raw(waker) };
                
                loop {
                    self.get_waker().register(&waker);
                    if self.scope.data.0.load(Ordering::Acquire) == 0 { break }
                    std::thread::park();
                }
            }
        }

        /// Creates a new scope for spawining scoped events.
        /// 
        /// The [`scope_async`](crate::scope_async) macro allows for the creation of `async` scopes, returning a [`Future`](std::future::Future)
        /// that completes when all the events spawned inside the scope have completed.
        /// 
        /// ```rust
        /// use blaze_rs::{buffer, scope_async, prelude::*};
        /// use futures::future::*;
        /// 
        /// let buffer = buffer![1, 2, 3, 4, 5]?;
        /// 
        /// let (left, right) = scope_async!(|s| async {
        ///     let left = buffer.read(s, ..2, None)?.join_async()?;
        ///     let right = buffer.read(s, ..2, None)?.join_async()?;
        ///     return try_join!(left, right);
        /// }).await?;
        /// 
        /// assert_eq!(left, vec![1, 2]);
        /// assert_eq!(right, vec![3, 4, 5]);
        /// # Ok::<_, Error>()
        /// ```
        /// 
        /// This macro can be called with the same form as [`scope`] or [`local_scope`].
        /// 
        /// ```rust
        /// use blaze_rs::{scope_async, prelude::*};
        /// use futures::future::*;
        /// 
        /// let ctx = SimpleContext::default()?;
        /// let buffer = Buffer::new_in(ctx, &[1, 2, 3, 4, 5], MemAccess::default(), false)?;
        /// 
        /// let (left, right) = scope_async!(buffer.context(), |s| async {
        ///     let left = buffer.read(s, ..2, None)?.join_async()?;
        ///     let right = buffer.read(s, ..2, None)?.join_async()?;
        ///     return try_join!(left, right);
        /// }).await?;
        /// 
        /// assert_eq!(left, vec![1, 2]);
        /// assert_eq!(right, vec![3, 4, 5]);
        /// # Ok::<_, Error>()
        /// ```
        /// 
        /// Unlike it's [blocking](local_scope) counterpart, [`scope_async`](crate::scope_async) does **not** ensure that all events inside the future
        /// will be ran. Rather, if the future is dropped before completion, it's destructor will block the current thread until every **already-started** event has completed,
        /// and discarting the remaining uninitialized events.
        /// 
        /// ```rust
        /// use blaze_rs::{buffer, scope_async, scope_async};
        /// use futures::{task::*, future::*};
        /// 
        /// let buffer = buffer![1, 2, 3, 4, 5]?;
        /// 
        /// let mut scope = Box::pin(scope_async!(|s| async {
        ///     let left = buffer.read(s, ..2, None)?.inspect(|_| println!("Left done!")).join_async()?.await;
        ///     let right = buffer.read(s, ..2, None)?.inspect(|_| println!("Right done!")).join_async()?.await;
        ///     return Ok((left, right));
        /// }));
        /// 
        /// let mut ctx = std::task::Context::from_waker(noop_waker_ref());
        /// let _ = scope.poll_unpin(&mut ctx)?;
        /// drop(scope); // prints "Left done!", doesn't print "Right done!"
        /// # Ok::<_, Error>()
        /// ```
        #[macro_export]
        macro_rules! scope_async {
            ($f:expr) => {
                $crate::scope_async!($crate::context::Global::get(), $f)
            };

            ($ctx:expr, $f:expr) => {
                async {
                    let scope = $crate::context::Scope::new_async($ctx);
                    unsafe {
                        $crate::context::InnerAsyncScope::new(&scope, $f).await
                    }
                }
            };
        }

        static TABLE : std::task::RawWakerVTable = std::task::RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

        unsafe fn clone_waker (ptr: *const ()) -> std::task::RawWaker {
            let thread = std::mem::ManuallyDrop::new(std::mem::transmute::<_, std::thread::Thread>(ptr));
            let ptr = std::mem::transmute(std::thread::Thread::clone(&thread));
            return std::task::RawWaker::new(ptr, &TABLE);
        }

        unsafe fn wake (ptr: *const ()) {
            let thread = std::mem::transmute::<_, std::thread::Thread>(ptr);
            thread.unpark();
        }
        
        unsafe fn wake_by_ref (ptr: *const ()) {
            let thread = std::mem::ManuallyDrop::new(std::mem::transmute::<_, std::thread::Thread>(ptr));
            thread.unpark();
        }

        unsafe fn drop_waker (ptr: *const ()) {
            let _ = std::mem::transmute::<_, std::thread::Thread>(ptr);
        }
    }

}