use std::{sync::{Arc, atomic::{AtomicUsize, Ordering, AtomicI32}}, marker::{PhantomData}, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}};
use opencl_sys::CL_SUCCESS;
use thinnbox::ThinBox;
use crate::{prelude::{Result, RawCommandQueue, RawEvent, Event, Error}, event::{consumer::{Consumer, Noop, NoopEvent, PhantomEvent}, EventStatus}};
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

struct ScopeData {
    items: AtomicUsize,
    err: AtomicI32
}

/// A scope to enqueue events in.\
/// See [`scope`] and [`local_scope`]
pub struct Scope<'scope, 'env: 'scope, C: 'env + Context = Global> {
    ctx: &'env C,
    data: Arc<ScopeData>,
    thread: ScopeWaker,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>
}

impl<'scope, 'env: 'scope, C: 'env + Context> Scope<'scope, 'env, C> {
    /// Creates a new [`Event`] scope targeted to `async` use
    #[docfg(feature = "futures")]
    #[inline(always)]
    pub unsafe fn new_async (ctx: &'env C) -> Self {
        Self::with_waker(ctx, Arc::new(futures::task::AtomicWaker::new()))
    }

    /// Creates a new `async` [`Event`] scope with the specified waker 
    #[docfg(feature = "futures")]
    #[inline(always)]
    fn with_waker (ctx: &'env C, waker: Arc<futures::task::AtomicWaker>) -> Self {
        Self {
            ctx,
            data: Arc::new(ScopeData {
                items: AtomicUsize::new(0),
                err: AtomicI32::new(CL_SUCCESS)
            }),
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

        if self.data.items.fetch_add(1, Ordering::AcqRel) == usize::MAX {
            panic!("too many items in scope")
        }

        if queue.size.fetch_add(1, Ordering::AcqRel) == usize::MAX {
            panic!("Queue size overflow");
        }

        let queue_size = queue.size.clone();
        let scope_data = self.data.clone();
        let scope_thread = self.thread.clone();

        // TODO HANDLE ERROR
        // ALSO, we are not getting into the callback
        evt.on_complete_silent(move |_, res| {
            let _ = queue_size.fetch_sub(1, Ordering::AcqRel);

            if let Err(e) = res {
                let _ = scope_data.err.compare_exchange(CL_SUCCESS, e.ty.as_i32(), Ordering::AcqRel, Ordering::Acquire);
            }

            Self::reduce_items(&scope_data, &scope_thread)
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

    /// Adds a callback function that will be executed when the event reaches the specified status.
    pub(crate) fn on_status<T: 'scope + Send, F: 'scope + Send + FnOnce(RawEvent, Result<EventStatus>) -> T, Cn: Consumer> (&'scope self, evt: &'env Event<Cn>, status: EventStatus, f: F) -> Result<crate::event::ScopedCallbackHandle<'scope, T>> {
        let (send, recv) = std::sync::mpsc::sync_channel::<_>(1);
        #[cfg(any(feature = "cl1_1", feature = "futures"))]
        let cb_data = std::sync::Arc::new(crate::event::CallbackHandleData {
            #[cfg(feature = "cl1_1")]
            flag: once_cell::sync::OnceCell::new(),
            #[cfg(feature = "futures")]
            waker: futures::task::AtomicWaker::new()
        });

        if self.data.items.fetch_add(1, Ordering::AcqRel) == usize::MAX {
            panic!("too many items in scope")
        }

        let my_data = self.data.clone();
        let my_thread = self.thread.clone();
        #[cfg(any(feature = "cl1_1", feature = "futures"))]
        let my_cb_data = cb_data.clone();

        let f = move |evt, status: Result<EventStatus>| {
            let f = std::panic::AssertUnwindSafe(|| f(evt, status.clone()));
            match send.send(std::panic::catch_unwind(f)) {
                Ok(_) => {
                    #[cfg(feature = "cl1_1")]
                    if let Some(flag) = my_cb_data.flag.get_or_init(|| None) {
                        flag.try_mark(status.err().map(|x| x.ty)).unwrap();
                    }
                    #[cfg(feature = "futures")]
                    my_cb_data.waker.wake();
                },
                Err(_) => {}
            }

            Self::reduce_items(&my_data, &my_thread)
        };

        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                let r#fn = ThinBox::<dyn 'scope + Send + FnMut(RawEvent, Result<EventStatus>)>::from_once(f);
            } else {
                let r#fn = unsafe { ThinBox::<dyn 'scope + Send + FnMut(RawEvent, Result<EventStatus>)>::from_once_unchecked(f) };
            }
        }
        let user_data = ThinBox::into_raw(r#fn);

        unsafe {
            if let Err(e) = evt.on_status_raw(status, crate::event::event_listener, user_data.as_ptr().cast()) {
                let _ = ThinBox::<dyn 'scope + Send + FnMut(RawEvent, Result<EventStatus>)>::from_raw(user_data); // drop user data
                Self::reduce_items(&self.data, &self.thread);
                return Err(e);
            }

            tri!(opencl_sys::clRetainEvent(evt.id()));
        }

        return Ok(crate::event::ScopedCallbackHandle { recv, #[cfg(any(feature = "cl1_1", feature = "futures"))] data: cb_data, phtm: PhantomData })
    }


    #[inline]
    fn reduce_items (scope_data: &ScopeData, scope_thread: &ScopeWaker) {
        if scope_data.items.fetch_sub(1, Ordering::AcqRel) == 1 {
            scope_thread.wake();
        }
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
    let data = ScopeData {
        items: AtomicUsize::new(0),
        err: AtomicI32::new(CL_SUCCESS)
    };

    let scope = Scope {
        ctx,
        data: Arc::new(data),
        thread: ScopeWaker::Thread(std::thread::current()),
        scope: PhantomData,
        env: PhantomData
    };

    // Run `f`, but catch panics so we can make sure to wait for all the threads to join.
    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));
    
    // Wait until all the events are finished.
    while scope.data.items.load(Ordering::Acquire) != 0 {
        std::thread::park();
    }

    // Throw any panic from `f`, or the return value of `f`.
    return match result {
        Err(e) => resume_unwind(e),
        Ok(x) => {
            let e = scope.data.err.load(Ordering::Acquire);
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
                if this.scope.data.items.load(Ordering::Acquire) != 0 {
                    return std::task::Poll::Pending;
                }

                // Complete
                match core::mem::replace(&mut this.fut, AsyncScopeFuture::Ended) {
                    AsyncScopeFuture::Panic(e) => resume_unwind(e),
                    AsyncScopeFuture::Value(x) => {
                        let e = this.scope.data.err.load(Ordering::Relaxed);
                        if e == CL_SUCCESS {
                            return std::task::Poll::Ready(x)
                        }
                        return std::task::Poll::Ready(Err(Error::from(e)));
                    },
                    AsyncScopeFuture::Ended => panic!("Scope already finished"),
                    #[cfg(debug_assertions)]
                    AsyncScopeFuture::Future(_) => unreachable!(),
                    #[cfg(not(debug_assertions))]
                    AsyncScopeFuture::Future(_) => unsafe { std::hint::unreachable_unchecked() }
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
                    if self.scope.data.items.load(Ordering::Acquire) == 0 { break }
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
                    let scope = unsafe { $crate::context::Scope::new_async($ctx) };
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