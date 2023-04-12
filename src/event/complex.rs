use super::{CallbackHandle, EventStatus, ProfilingInfo, RawEvent, ScopedCallbackHandle};
use crate::prelude::*;
use blaze_proc::docfg;
use opencl_sys::*;
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    panic::{AssertUnwindSafe, UnwindSafe},
    time::{Duration, SystemTime},
};

/// A dynamic event that **can** be shared between threads.
pub type DynEvent<'a, T> = Event<Box<dyn 'a + Consumer<Output = T> + Send>>;
/// A dynamic event that **cannot** be shared between threads.
pub type DynLocalEvent<'a, T> = Event<Box<dyn 'a + Consumer<Output = T>>>;

pub(crate) mod ext {
    use crate::event::_consumer::*;
    use crate::event::*;
    use blaze_proc::docfg;
    use std::marker::PhantomData;
    use std::panic::AssertUnwindSafe;

    /// Event with no underlying operations.
    pub type NoopEvent = Event<Noop>;
    /// Event with a [`PhantomData`] consumer.
    pub type PhantomEvent<T> = Event<PhantomData<T>>;
    /// Event for [`specific`](super::Event::specific).
    pub type SpecificEvent<'a, C> = Event<Specific<'a, C>>;
    /// Event for [`abortable`](super::Event::abortable).
    #[docfg(feature = "cl1_1")]
    pub type AbortableEvent<C> = Event<abort::Abort<C>>;
    /// Event for [`map`](super::Event::map).
    pub type MapEvent<T, C, F> = Event<Map<T, C, F>>;
    /// Event for [`try_map`](super::Event::try_map).
    pub type TryMapEvent<T, C, F> = Event<TryMap<T, C, F>>;
    /// Event for [`catch_unwind`](super::Event::catch_unwind).
    pub type CatchUnwindEvent<C> = Event<CatchUnwind<C>>;
    /// Event for [`assert_catch_unwind`](super::Event::assert_catch_unwind).
    pub type AssertCatchUnwindEvent<C> = CatchUnwindEvent<AssertUnwindSafe<C>>;
    /// Event for [`flatten_result`](super::Event::flatten_result).
    pub type FlattenResultEvent<C> = Event<FlattenResult<C>>;
    /// Event for [`inspect`](super::Event::flatten).
    pub type InspectEvent<C, F> = Event<Inspect<C, F>>;
    /// Event for [`flatten`](super::Event::flatten_scoped).
    #[docfg(feature = "cl1_1")]
    pub type FlattenEvent<C> = Event<Flatten<C>>;
    /// Event for [`flatten_scoped`](super::Event::flatten_scoped).
    #[docfg(feature = "cl1_1")]
    pub type FlattenScopedEvent<'a, C> = Event<FlattenScoped<'a, C>>;
    /// Event for [`join_all`](super::Event::join_all).
    #[docfg(feature = "cl1_1")]
    pub type JoinAllEvent<C> = Event<JoinAll<C>>;
}

use super::consumer::*;

/// An event with a consumer that will be executed on the completion of the former.
///
/// When using OpenCL 1.0, the event will also contain a sender that will send the event's callbacks,
/// (like [`on_complete`](Event::on_complete)) to a different thread to be executed acordingly.
#[derive(Debug, Clone)]
pub struct Event<C> {
    inner: RawEvent,
    consumer: C,
    #[cfg(not(feature = "cl1_1"))]
    send: std::sync::Arc<utils_atomics::FillQueue<super::listener::EventCallback>>,
    #[cfg(feature = "cl1_1")]
    send: std::marker::PhantomData<
        std::sync::Arc<
            utils_atomics::FillQueue<(
                RawEvent,
                EventStatus,
                Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send + Sync>,
            )>,
        >,
    >,
}

impl NoopEvent {
    /// Creates a new noop event.
    #[inline(always)]
    pub fn new_noop(inner: RawEvent) -> Self {
        Self::new(inner, Noop)
    }

    /// Adds a consumer to a noop event
    #[inline(always)]
    pub fn set_consumer<C: Consumer>(self, consumer: C) -> Event<C> {
        Event {
            inner: self.inner,
            consumer,
            send: self.send,
        }
    }
}

impl<'a, T: 'a> PhantomEvent<T> {
    /// Creates a new phantom event.
    #[inline(always)]
    pub fn new_phantom(inner: RawEvent) -> Self {
        Self::new(inner, PhantomData)
    }
}

impl<C: Consumer> Event<C> {
    /// Creates a new event with the specified consumer.   
    #[inline(always)]
    pub fn new(inner: RawEvent, consumer: C) -> Self {
        Self {
            inner,
            consumer,
            #[cfg(not(feature = "cl1_1"))]
            send: super::listener::get_sender(),
            #[cfg(feature = "cl1_1")]
            send: std::marker::PhantomData,
        }
    }

    /// Consumes the event and returns it's inner parts
    #[inline(always)]
    pub fn into_parts(self) -> (RawEvent, C) {
        (self.inner, self.consumer)
    }

    /// Returns a reference to the underlying [`RawEvent`].
    #[inline(always)]
    pub fn as_raw(&self) -> &RawEvent {
        &self.inner
    }

    /// Maps the current event consumer to a new one
    #[inline(always)]
    pub fn map_consumer<'a, N: 'a + Consumer, F: FnOnce(C) -> N>(this: Self, f: F) -> Event<N>
    where
        C: 'a,
    {
        Event {
            inner: this.inner,
            consumer: f(this.consumer),
            send: this.send,
        }
    }

    /// Takes the current event's [`Consumer`], leaving a [`Noop`] in it's place.
    ///
    /// # Safety
    /// This method is unsafe because the [`Consumer`] may have implementation details needed for the general safety of the event.
    #[inline]
    pub unsafe fn take_consumer(self) -> (C, NoopEvent) {
        let consumer = self.consumer;
        let evt = Event {
            inner: self.inner,
            consumer: Noop,
            send: self.send,
        };

        return (consumer, evt);
    }

    /// Makes the current event abortable.
    /// When aborted, the event will not be unqueued from the OpenCL queue, rather the Blaze event will return early with a result of `Ok(None)`.
    /// If the event isn't aborted before it's completion, it will return `Ok(Some(value))` in case of success, and `Err(error)` if it fails.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn abortable(self) -> Result<(AbortableEvent<C>, super::AbortHandle)> {
        let ctx = self.raw_context()?;
        let flag = super::FlagEvent::new_in(&ctx)?;
        let aborted = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(super::abort::UNINIT));

        let my_flag = flag.clone();
        let my_aborted = aborted.clone();
        self.on_complete_silent(move |_, res| {
            let res = match res {
                Ok(_) => None,
                Err(e) => Some(e.ty),
            };

            match my_flag.try_mark(res) {
                Ok(true) => {
                    my_aborted.store(super::abort::FALSE, std::sync::atomic::Ordering::Release)
                }
                _ => {}
            }
        })?;

        let handle = super::AbortHandle {
            inner: flag.clone(),
            aborted: aborted.clone(),
        };

        let consumer = Abort {
            aborted,
            consumer: self.consumer,
        };

        let event = AbortableEvent::new(flag.into_inner(), consumer);
        return Ok((event, handle));
    }

    /// Returns an event with the consumer restricted to a specified lifetime.
    ///
    /// The original consumer must have a lifetime greater or equal to the new one.
    #[inline(always)]
    pub fn specific<'a>(self) -> SpecificEvent<'a, C>
    where
        C: 'a,
    {
        Event {
            inner: self.inner,
            consumer: Specific::new(self.consumer),
            send: self.send,
        }
    }

    /// Returns an event that maps the result of the previous event.
    #[inline(always)]
    pub fn map<F: FnOnce(C::Output) -> U, U>(self, f: F) -> MapEvent<C::Output, C, F> {
        Event {
            inner: self.inner,
            consumer: Map::new(self.consumer, f),
            send: self.send,
        }
    }

    /// Returns an event that maps the result of the previous event, flattening the result.
    #[inline(always)]
    pub fn try_map<F: FnOnce(C::Output) -> Result<U>, U>(
        self,
        f: F,
    ) -> TryMapEvent<C::Output, C, F> {
        Event {
            inner: self.inner,
            consumer: TryMap::new(self.consumer, f),
            send: self.send,
        }
    }

    /// Returns an event that will catch the consumer's panic.
    /// Note that this method requires the current consumer to be [`UnwindSafe`].
    /// If this requirement proves bothersome, you can use [`assert_unwind_safe`](Event::assert_catch_unwind).
    #[inline(always)]
    pub fn catch_unwind(self) -> CatchUnwindEvent<C>
    where
        C: UnwindSafe,
    {
        CatchUnwindEvent {
            inner: self.inner,
            consumer: CatchUnwind(self.consumer),
            send: self.send,
        }
    }

    /// Returns an event that will catch the consumer's panic.
    /// Note that this method does **not** requires the current consumer to be [`UnwindSafe`], as it's wrapped with [`AssertUnwindSafe`].
    /// If the consumer is known to be [`UnwindSafe`], the [`catch_unwind`](Event::catch_unwind) method is preferable.
    #[inline(always)]
    pub fn assert_catch_unwind(self) -> AssertCatchUnwindEvent<C> {
        AssertCatchUnwindEvent {
            inner: self.inner,
            consumer: CatchUnwind(AssertUnwindSafe(self.consumer)),
            send: self.send,
        }
    }

    /// Returns an event that will inspect it's parent's return value before completing.
    #[inline(always)]
    pub fn inspect<F: FnOnce(&C::Output)>(self, f: F) -> InspectEvent<C, F> {
        InspectEvent {
            inner: self.inner,
            consumer: Inspect(self.consumer, f),
            send: self.send,
        }
    }

    #[inline(always)]
    pub(super) unsafe fn consume(self) -> Result<C::Output> {
        self.consumer.consume()
    }
}

impl<T, C: Consumer<Output = Result<T>>> Event<C> {
    /// Returns an event that flattens the result of it's parent.
    #[inline(always)]
    pub fn flatten_result(self) -> FlattenResultEvent<C> {
        FlattenResultEvent {
            inner: self.inner,
            consumer: FlattenResult(self.consumer),
            send: self.send,
        }
    }
}

impl<N: Consumer, C: Consumer<Output = Event<N>>> Event<C> {
    #[docfg(feature = "cl1_1")]
    pub fn flatten(self) -> Result<FlattenEvent<N>>
    where
        C: 'static + Send + Sync,
        N: 'static + Send,
    {
        use super::FlagEvent;

        let ctx = self.raw_context()?;
        let flag = FlagEvent::new_in(&ctx)?;
        let sub = flag.subscribe();

        let cb = self.then_result(move |evt| match evt {
            Ok(evt) => unsafe {
                let (consumer, evt) = evt.take_consumer();
                let my_flag = flag.clone();
                return match evt.on_complete_silent(move |_, status| {
                    my_flag.try_mark(status.err().map(|x| x.ty)).unwrap();
                }) {
                    Ok(_) => Result::Ok(consumer),
                    Err(e) => {
                        flag.try_mark(Some(e.ty))?;
                        Result::Err(e)
                    }
                };
            },

            Err(e) => {
                flag.try_mark(Some(e.ty))?;
                return Result::Err(e);
            }
        })?;

        return Ok(Event::new(sub, FlattenScoped(cb)));
    }

    #[docfg(feature = "cl1_1")]
    pub fn flatten_scoped<'scope, 'env, Ctx: Context>(
        self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
    ) -> Result<FlattenScopedEvent<'scope, N>>
    where
        C: 'scope + Send,
        N: 'scope + Send,
    {
        use super::FlagEvent;

        let ctx = self.raw_context()?;
        let flag = FlagEvent::new_in(&ctx)?;
        let sub = flag.subscribe();

        let cb = self.then_result_scoped(scope, move |evt| match evt {
            Ok(evt) => unsafe {
                let (consumer, evt) = evt.take_consumer();
                let my_flag = flag.clone();
                return match evt.on_complete_silent(move |_, status| {
                    my_flag.try_mark(status.err().map(|x| x.ty)).unwrap();
                }) {
                    Ok(_) => Result::Ok(consumer),
                    Err(e) => {
                        flag.try_mark(Some(e.ty))?;
                        Result::Err(e)
                    }
                };
            },

            Err(e) => {
                flag.try_mark(Some(e.ty))?;
                return Result::Err(e);
            }
        })?;

        return Ok(Event::new(sub, FlattenScoped(cb)));
    }

    #[inline(always)]
    pub fn flatten_join(self) -> Result<N::Output> {
        return self.join()?.join();
    }

    #[docfg(feature = "futures")]
    #[inline(always)]
    pub async fn flatten_join_async(self) -> Result<N::Output>
    where
        C: Unpin,
        N: Unpin,
    {
        return self.join_async()?.await?.join_async()?.await;
    }
}

impl<'a, C: 'a + Consumer> Event<C> {
    /// Turn's the event into a [`DynEvent`].
    /// A [`DynEvent`] contains a boxed [dynamic](https://doc.rust-lang.org/stable/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait) consumer that **can** be shared between threads.
    #[inline(always)]
    pub fn into_dyn(self) -> DynEvent<'a, C::Output>
    where
        C: Send,
    {
        DynEvent {
            inner: self.inner,
            consumer: Box::new(self.consumer),
            send: self.send,
        }
    }

    /// Turn's the event into a [`DynLocalEvent`].
    /// A [`DynLocalEvent`] contains a boxed [dynamic](https://doc.rust-lang.org/stable/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait) consumer that **cannot** be shared between threads.
    #[inline(always)]
    pub fn into_local(self) -> DynLocalEvent<'a, C::Output> {
        DynLocalEvent {
            inner: self.inner,
            consumer: Box::new(self.consumer),
            send: self.send,
        }
    }

    /// Blocks the current thread until the event has completed, consuming it and returning it's value.
    #[inline(always)]
    pub fn join(self) -> Result<C::Output> {
        self.join_by_ref()?;
        // SAFETY: Event has already been completed
        unsafe { self.consume() }
    }

    /// Blocks the current thread until the event has completes, consuming it and returning it's value, alongside it's profiling info in nanoseconds.
    #[inline]
    pub fn join_with_nanos(self) -> Result<(C::Output, ProfilingInfo<u64>)> {
        self.join_by_ref()?;
        let nanos = self.profiling_nanos()?;
        // SAFETY: Event has already been completed
        let v = unsafe { self.consume()? };
        Ok((v, nanos))
    }

    /// Blocks the current thread until the event has completes, consuming it and returning it's value, alongside it's profiling info in [`SystemTime`].
    #[inline]
    pub fn join_with_time(self) -> Result<(C::Output, ProfilingInfo<SystemTime>)> {
        self.join_by_ref()?;
        let nanos = self.profiling_time()?;
        // SAFETY: Event has already been completed
        let v = unsafe { self.consume()? };
        Ok((v, nanos))
    }

    /// Blocks the current thread until the event has completes, consuming it and returning it's value, alongside it's duration.
    #[inline]
    pub fn join_with_duration(self) -> Result<(C::Output, Duration)> {
        self.join_by_ref()?;
        let nanos = self.duration()?;
        // SAFETY: Event has already been completed
        let v = unsafe { self.consume()? };
        Ok((v, nanos))
    }

    /// Blocks the current thread util the event has completed, consuming it and returning it's value if it completed correctly, and panicking otherwise.
    #[inline(always)]
    pub fn join_unwrap(self) -> C::Output {
        self.join().unwrap()
    }

    /// Returns a future that waits for the event to complete without blocking.
    #[inline(always)]
    #[docfg(feature = "futures")]
    pub fn join_async(self) -> Result<crate::event::EventWait<C>>
    where
        C: Unpin,
    {
        crate::event::EventWait::new(self)
    }

    /// Returns an event that completes when all the events inside `iter` complete (or one of them fails).
    /// The new event will return it's parents results inside a [`Vec`], in the same order they were in the iterator.\
    /// Note that if the iterator is empty, this funtion will return an error.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn join_all<I: IntoIterator<Item = Self>>(iter: I) -> Result<JoinAllEvent<C>> {
        let (raw, consumers) = iter
            .into_iter()
            .map(|x| (x.inner, x.consumer))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        if raw.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidEventWaitList,
                "no events inside the iterator",
            ));
        }

        let queue = raw[0]
            .command_queue()?
            .ok_or_else(|| Error::new(ErrorKind::InvalidCommandQueue, "command queue not found"))?;

        let barrier = queue.barrier(Some(&raw))?;
        return Ok(Event::new(barrier, JoinAll(consumers)));
    }

    /// Returns an event that completes when all the events inside `iter` complete (or one of them fails).
    /// The new event will return it's parents results inside a [`Vec`], in the same order they were in the iterator.\
    /// Note that if the iterator is empty, this funtion will return an error.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(all(feature = "cl1_1", not(feature = "cl1_2")))]
    #[inline(always)]
    pub fn join_all<I: IntoIterator<Item = Self>>(iter: I) -> Result<JoinAllEvent<C>> {
        let mut iter = iter.into_iter().peekable();
        let size = std::sync::Arc::new(std::sync::atomic::AtomicUsize::default());

        let mut consumers = Vec::with_capacity(match iter.size_hint() {
            (_, Some(len)) => len,
            (len, _) => len,
        });

        let ctx = match iter.peek() {
            Some(evt) => evt.raw_context()?,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidEventWaitList,
                    "no events inside the iterator",
                ))
            }
        };

        let flag = super::FlagEvent::new_in(&ctx)?.into_inner();

        for evt in iter.into_iter() {
            let flag = flag.clone();
            let size = size.clone();
            size.fetch_add(1, std::sync::atomic::Ordering::AcqRel);

            evt.on_complete(move |_, err| unsafe {
                if let Err(e) = err {
                    clSetUserEventStatus(flag.id(), e.ty.as_i32());
                    return;
                }

                if size.fetch_sub(1, std::sync::atomic::Ordering::AcqRel) == 1 {
                    clSetUserEventStatus(flag.id(), CL_COMPLETE);
                    return;
                }
            })?;

            consumers.push(evt.consumer);
        }

        return Ok(Event::new(flag, JoinAll(consumers)));
    }

    /// Blocks the current thread until all the events in the iterator have completed, returning their values inside a [`Vec`].
    /// The order of the values in the result is the same as their parents inside the iterator.
    #[inline(always)]
    pub fn join_all_blocking<I: IntoIterator<Item = Self>>(iter: I) -> Result<Vec<C::Output>> {
        let (raw, consumers) = iter
            .into_iter()
            .map(|x| (x.inner, x.consumer))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        RawEvent::join_all_by_ref(&raw)?;
        return consumers
            .into_iter()
            .map(|x| unsafe { x.consume() })
            .try_collect();
    }

    /// Blocks the current thread until all the events in the array have completed, returning their values in a new array.
    /// The order of the values in the result is the same as their parents inside the iterator.
    #[inline(always)]
    pub fn join_sized_blocking<const N: usize>(iter: [Self; N]) -> Result<[C::Output; N]> {
        let mut raw = MaybeUninit::uninit_array::<N>();
        let mut consumers = MaybeUninit::uninit_array::<N>();

        unsafe {
            for (i, event) in iter.into_iter().enumerate() {
                raw.get_unchecked_mut(i).write(event.inner);
                consumers.get_unchecked_mut(i).write(event.consumer);
            }

            let raw = MaybeUninit::array_assume_init(raw);
            let consumers = MaybeUninit::array_assume_init(consumers);

            RawEvent::join_all_by_ref(&raw)?;
            return consumers.try_map(|x| x.consume());
        }
    }
}

impl<C: Consumer> Event<C> {
    #[inline]
    pub fn then<T: 'static + Send, F: 'static + Send + Sync + FnOnce(C::Output) -> T>(
        self,
        f: F,
    ) -> Result<CallbackHandle<Result<T>>>
    where
        C: 'static + Send + Sync,
    {
        let (consumer, this) = unsafe { self.take_consumer() };
        let f = move |_, status: Result<EventStatus>| match status {
            Ok(_) => unsafe { consumer.consume().map(f) },
            Err(e) => Err(e),
        };
        return this.on_complete(f);
    }

    #[inline]
    pub fn then_scoped<
        'scope,
        'env,
        T: 'scope + Send,
        F: 'scope + Send + Sync + FnOnce(C::Output) -> T,
        Ctx: Context,
    >(
        self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
        f: F,
    ) -> Result<ScopedCallbackHandle<'scope, Result<T>>>
    where
        C: 'scope + Send,
    {
        let (consumer, this) = unsafe { self.take_consumer() };
        let f = move |_, status: Result<EventStatus>| match status {
            Ok(_) => unsafe { consumer.consume().map(f) },
            Err(e) => Err(e),
        };
        return unsafe {
            core::mem::transmute::<&Event<Noop>, &'env NoopEvent>(&this)
                .on_complete_scoped(scope, f)
        };
    }

    #[inline]
    pub fn then_result<
        T: 'static + Send,
        F: 'static + Send + Sync + FnOnce(Result<C::Output>) -> T,
    >(
        self,
        f: F,
    ) -> Result<CallbackHandle<T>>
    where
        C: 'static + Send + Sync,
    {
        let (consumer, this) = unsafe { self.take_consumer() };
        let f = move |_, status: Result<EventStatus>| {
            f(status.and_then(|_| unsafe { consumer.consume() }))
        };
        return this.on_complete(f);
    }

    #[inline]
    pub fn then_result_scoped<
        'scope,
        'env,
        T: 'scope + Send,
        F: 'scope + Send + FnOnce(Result<C::Output>) -> T,
        Ctx: Context,
    >(
        self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
        f: F,
    ) -> Result<ScopedCallbackHandle<'scope, T>>
    where
        C: 'scope + Send,
    {
        let (consumer, this) = unsafe { self.take_consumer() };
        let f = move |_, status: Result<EventStatus>| {
            f(status.and_then(|_| unsafe { consumer.consume() }))
        };
        return unsafe {
            core::mem::transmute::<_, &'env NoopEvent>(&this).on_complete_scoped(scope, f)
        };
    }

    #[inline(always)]
    pub fn on_submit<
        T: 'static + Send,
        F: 'static + Send + Sync + FnOnce(RawEvent, Result<EventStatus>) -> T,
    >(
        &self,
        f: F,
    ) -> Result<CallbackHandle<T>> {
        self.on_status(EventStatus::Submitted, f)
    }

    #[inline(always)]
    pub fn on_run<
        T: 'static + Send,
        F: 'static + Send + Sync + FnOnce(RawEvent, Result<EventStatus>) -> T,
    >(
        &self,
        f: F,
    ) -> Result<CallbackHandle<T>> {
        self.on_status(EventStatus::Running, f)
    }

    #[inline(always)]
    pub fn on_complete<
        T: 'static + Send,
        F: 'static + Send + Sync + FnOnce(RawEvent, Result<EventStatus>) -> T,
    >(
        &self,
        f: F,
    ) -> Result<CallbackHandle<T>> {
        self.on_status(EventStatus::Complete, f)
    }

    /// TODO DOC
    pub fn on_status<
        T: 'static + Send,
        F: 'static + Send + Sync + FnOnce(RawEvent, Result<EventStatus>) -> T,
    >(
        &self,
        status: EventStatus,
        f: F,
    ) -> Result<CallbackHandle<T>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                return RawEvent::on_status(&self, status, f)
            } else {
                let (send, recv) = std::sync::mpsc::sync_channel(1);
                #[cfg(feature = "futures")]
                let data = std::sync::Arc::new(super::CallbackHandleData {
                    waker: futures::task::AtomicWaker::new()
                });
                #[cfg(feature = "futures")]
                let my_data = data.clone();

                self.on_status_silent(status, move |evt, status| {
                    let f = std::panic::AssertUnwindSafe(|| f(evt, status));
                    match send.send(std::panic::catch_unwind(f)) {
                        Ok(_) => {
                            #[cfg(feature = "futures")]
                            my_data.waker.wake();
                        },
                        Err(_) => {}
                    }
                })?;

                return Ok(CallbackHandle { recv, #[cfg(feature = "futures")] data, phtm: PhantomData })
            }
        }
    }

    #[inline(always)]
    pub fn on_submit_scoped<
        'scope,
        'env,
        T: 'scope + Send,
        F: 'scope + Send + FnOnce(RawEvent, Result<EventStatus>) -> T,
        Ctx: Context,
    >(
        &'env self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
        f: F,
    ) -> Result<ScopedCallbackHandle<'scope, T>> {
        self.on_status_scoped(scope, EventStatus::Submitted, f)
    }

    #[inline(always)]
    pub fn on_run_scoped<
        'scope,
        'env,
        T: 'scope + Send,
        F: 'scope + Send + FnOnce(RawEvent, Result<EventStatus>) -> T,
        Ctx: Context,
    >(
        &'env self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
        f: F,
    ) -> Result<ScopedCallbackHandle<'scope, T>> {
        self.on_status_scoped(scope, EventStatus::Running, f)
    }

    #[inline(always)]
    pub fn on_complete_scoped<
        'scope,
        'env,
        T: 'scope + Send,
        F: 'scope + Send + FnOnce(RawEvent, Result<EventStatus>) -> T,
        Ctx: Context,
    >(
        &'env self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
        f: F,
    ) -> Result<ScopedCallbackHandle<'scope, T>> {
        self.on_status_scoped(scope, EventStatus::Complete, f)
    }

    /// TODO DOCS
    #[inline(always)]
    pub fn on_status_scoped<
        'scope,
        'env,
        T: 'scope + Send,
        F: 'scope + Send + FnOnce(RawEvent, Result<EventStatus>) -> T,
        Ctx: Context,
    >(
        &'env self,
        scope: &'scope Scope<'scope, 'env, Ctx>,
        status: EventStatus,
        f: F,
    ) -> Result<ScopedCallbackHandle<'scope, T>> {
        scope.on_status(self, status, f)
    }

    /// Adds a callback function that will be executed when the event is submitted.
    #[inline(always)]
    pub fn on_submit_silent(
        &self,
        f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send + Sync,
    ) -> Result<()> {
        self.on_status_silent(EventStatus::Submitted, f)
    }

    /// Adds a callback function that will be executed when the event starts running.
    #[inline(always)]
    pub fn on_run_silent(
        &self,
        f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send + Sync,
    ) -> Result<()> {
        self.on_status_silent(EventStatus::Running, f)
    }

    /// Adds a callback function that will be executed when the event completes.
    #[inline(always)]
    pub fn on_complete_silent(
        &self,
        f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send + Sync,
    ) -> Result<()> {
        self.on_status_silent(EventStatus::Complete, f)
    }

    /// Registers a user callback function for a specific command execution status.\
    /// The registered callback function will be called when the execution status of command associated with event changes to an execution status equal to or past the status specified by `status`.\
    /// Each call to [`Event::on_status`] registers the specified user callback function on a callback stack associated with event. The order in which the registered user callback functions are called is undefined.\
    /// All callbacks registered for an event object must be called before the event object is destroyed. Callbacks should return promptly.\
    /// Behavior is undefined when calling expensive system routines, OpenCL APIs to create contexts or command-queues, or blocking OpenCL APIs in an event callback. Rather than calling a blocking OpenCL API in an event callback, applications may call a non-blocking OpenCL API, then register a completion callback for the non-blocking OpenCL API with the remainder of the work.\
    /// Because commands in a command-queue are not required to begin execution until the command-queue is flushed, callbacks that enqueue commands on a command-queue should either call [`RawCommandQueue::flush`] on the queue before returning, or arrange for the command-queue to be flushed later.
    #[inline(always)]
    pub fn on_status_silent(
        &self,
        status: EventStatus,
        f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send + Sync,
    ) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                return RawEvent::on_status_silent(&self, status, f)
            } else {
                let cb = super::listener::EventCallback { evt: self.inner.clone(), status, cb: super::listener::Callback::Boxed(Box::new(f)) };
                self.send.push(cb);
                return Ok(())
            }
        }
    }

    #[inline(always)]
    pub unsafe fn on_submit_raw(
        &self,
        f: unsafe extern "C" fn(
            event: cl_event,
            event_command_status: cl_int,
            user_data: *mut c_void,
        ),
        user_data: *mut c_void,
    ) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Submitted, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_run_raw(
        &self,
        f: unsafe extern "C" fn(
            event: cl_event,
            event_command_status: cl_int,
            user_data: *mut c_void,
        ),
        user_data: *mut c_void,
    ) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Running, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_complete_raw(
        &self,
        f: unsafe extern "C" fn(
            event: cl_event,
            event_command_status: cl_int,
            user_data: *mut c_void,
        ),
        user_data: *mut c_void,
    ) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Complete, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_status_raw(
        &self,
        status: EventStatus,
        f: unsafe extern "C" fn(
            event: cl_event,
            event_command_status: cl_int,
            user_data: *mut c_void,
        ),
        user_data: *mut c_void,
    ) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                return RawEvent::on_status_raw(&self, status, f, user_data)
            } else {
                let cb = super::listener::EventCallback { evt: self.inner.clone(), status, cb: super::listener::Callback::Raw(f, user_data) };
                self.send.push(cb);
                return Ok(())
            }
        }
    }
}

impl<C: Consumer> Deref for Event<C> {
    type Target = RawEvent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

//impl<C: Unpin> Unpin for Event<C> {}
