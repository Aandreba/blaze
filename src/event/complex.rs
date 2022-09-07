use std::{sync::mpsc::{channel, Sender}, ops::Deref, ffi::c_void, time::{SystemTime, Duration}, marker::PhantomData, mem::MaybeUninit};
use opencl_sys::*;
use blaze_proc::docfg;
use crate::{prelude::*};
use super::{RawEvent, EventStatus, ProfilingInfo, Consumer, Noop};

pub type DynEvent<'a, T> = Event<T, Box<dyn Consumer<'a, T>>>;
pub type NoopEvent<'a> = Event<(), Noop::<'a>>;

pub struct Event<T, C> {
    inner: RawEvent,
    consumer: C,
    #[cfg(not(feature = "cl1_1"))]
    send: Sender<super::listener::EventCallback>,
    phtm: PhantomData<T>
}

impl<'a> NoopEvent<'a> {
    #[inline(always)]
    pub(crate) fn new_noop (inner: RawEvent) -> Self {
        Self::new(inner, Noop::new())
    }
}

impl<'a, T, C: Consumer<'a, T>> Event<T, C> {
    #[inline(always)]
    pub(crate) fn new (inner: RawEvent, consumer: C) -> Self {
        #[cfg(not(feature = "cl1_1"))]
        let (send, recv) = channel();
        #[cfg(not(feature = "cl1_1"))]
        let list = super::listener::get_sender();
        #[cfg(not(feature = "cl1_1"))]
        list.send((inner.clone(), recv)).unwrap();

        Self {
            inner,
            consumer,
            #[cfg(not(feature = "cl1_1"))]
            send,
            phtm: PhantomData
        }
    }

    #[inline(always)]
    pub fn into_dyn (self) -> DynEvent<'a, T> {
        DynEvent {
            inner: self.inner,
            consumer: Box::new(self.consumer),
            #[cfg(not(feature = "cl1_1"))]
            send: self.send,
            phtm: self.phtm
        }
    }

    #[inline(always)]
    pub(super) fn consume (self) -> Result<T> {
        self.consumer.consume()
    }
}

impl<'a, T, C: Consumer<'a, T>> Event<T, C> {
    #[inline(always)]
    pub fn join (self) -> Result<T> {
        self.join_by_ref()?;
        self.consume()
    }

    #[inline]
    pub fn join_with_nanos (self) -> Result<(T, ProfilingInfo<u64>)> {
        self.join_by_ref()?;
        let nanos = self.profiling_nanos()?;
        let v = self.consume()?;
        Ok((v, nanos))
    }

    #[inline]
    pub fn join_with_time (self) -> Result<(T, ProfilingInfo<SystemTime>)> {
        self.join_by_ref()?;
        let nanos = self.profiling_time()?;
        let v = self.consume()?;
        Ok((v, nanos))
    }

    #[inline]
    pub fn join_with_duration (self) -> Result<(T, Duration)> {
        self.join_by_ref()?;
        let nanos = self.duration()?;
        let v = self.consume()?;
        Ok((v, nanos))
    }

    /// Blocks the current thread util the event has completed, returning `data` if it completed correctly, and panicking otherwise.
    #[inline(always)]
    pub fn join_unwrap (self) -> T {
        self.join().unwrap()
    }

    /// Returns a future that waits for the event to complete without blocking.
    #[inline(always)]
    #[docfg(feature = "futures")]
    pub fn join_async (self) -> Result<crate::event::EventWait<T, C>> {
        crate::event::EventWait::new(self)
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn join_all<I: IntoIterator<Item = Self>> (iter: I) -> Result<JoinAll<T, C>> {
        let (raw, consumers) = iter.into_iter()
            .map(|x| (x.inner, x.consumer))
            .unzip::<_, _, Vec<_>, Vec<_>>();
        
        if raw.is_empty() {
            return Err(Error::new(ErrorType::InvalidEventWaitList, "no events inside the iterator"));
        }

        let queue = raw[0].command_queue()?
            .ok_or_else(|| Error::new(ErrorType::InvalidCommandQueue, "command queue not found"))?;

        let barrier = queue.barrier(&raw)?;
        return Ok(Event::new(barrier, JoinAllConsumer(consumers)));
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
    #[cfg(all(feature = "cl1_1", not(feature = "cl1_2")))]
    #[inline(always)]
    pub fn join_all<I: IntoIterator<Item = Self>> (iter: I) -> Result<JoinAll<T, C>> {
        let mut iter = iter.into_iter().peekable();
        let mut size = crate::context::Size::new();
        let mut consumers = Vec::with_capacity(match iter.size_hint() {
            (_, Some(len)) => len,
            (len, _) => len
        });

        let ctx = match iter.peek() {
            Some(evt) => evt.raw_context()?,
            None => return Err(Error::new(ErrorType::InvalidEventWaitList, "no events inside the iterator"))
        };

        let mut flag = super::FlagEvent::new_in(&ctx)?.into_inner();

        for evt in iter.into_iter() {
            let flag = flag.clone();
            let size = size.clone();

            evt.on_complete(move |_, err| unsafe {
                if let Err(e) = err {
                    clSetUserEventStatus(flag.id(), e.ty as i32);
                    return;
                }

                if size.drop_last() {
                    clSetUserEventStatus(flag.id(), CL_COMPLETE);
                    return;
                }
            })?;

            consumers.push(evt.consumer);
        }

        return Ok(Event::new(flag, JoinAllConsumer(consumers)));
    }

    /// Blocks the current thread until all the events in the iterator have completed, returning their values.
    #[inline(always)]
    pub fn join_all_blocking<I: IntoIterator<Item = Self>> (iter: I) -> Result<Vec<T>> {
        let (raw, consumers) = iter.into_iter()
            .map(|x| (x.inner, x.consumer))
            .unzip::<_, _, Vec<_>, Vec<_>>();
        
        RawEvent::join_all_by_ref(&raw)?;
        return consumers.into_iter().map(Consumer::consume).try_collect()
    }

    /// Blocks the current thread until all the events in the iterator have completed, returning their values.
    #[inline(always)]
    pub fn join_all_sized_blocking<const N: usize> (iter: [Self; N]) -> Result<[T; N]> {
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
            return consumers.try_map(Consumer::consume);
        }
    }
}

impl<'a, T, C: Consumer<'a, T>> Event<T, C> {
    /// Adds a callback function that will be executed when the event is submitted.
    #[inline(always)]
    pub fn on_submit (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Submitted, f)
    }

    /// Adds a callback function that will be executed when the event starts running.
    #[inline(always)]
    pub fn on_run (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Running, f)
    }

    /// Adds a callback function that will be executed when the event completes.
    #[inline(always)]
    pub fn on_complete (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status(EventStatus::Complete, f)
    }

    /// Registers a user callback function for a specific command execution status.\
    /// The registered callback function will be called when the execution status of command associated with event changes to an execution status equal to or past the status specified by `status`.\
    /// Each call to [`RawEvent::on_status`] registers the specified user callback function on a callback stack associated with event. The order in which the registered user callback functions are called is undefined.\
    /// All callbacks registered for an event object must be called before the event object is destroyed. Callbacks should return promptly.\
    /// Behavior is undefined when calling expensive system routines, OpenCL APIs to create contexts or command-queues, or blocking OpenCL APIs in an event callback. Rather than calling a blocking OpenCL API in an event callback, applications may call a non-blocking OpenCL API, then register a completion callback for the non-blocking OpenCL API with the remainder of the work.\
    /// Because commands in a command-queue are not required to begin execution until the command-queue is flushed, callbacks that enqueue commands on a command-queue should either call [`RawCommandQueue::flush`] on the queue before returning, or arrange for the command-queue to be flushed later.
    #[inline(always)]
    pub fn on_status (&self, status: EventStatus, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status_boxed(status, Box::new(f) as Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>)
    }

    #[inline(always)]
    pub fn on_submit_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Submitted, f)
    }

    #[inline(always)]
    pub fn on_run_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Running, f)
    }

    #[inline(always)]
    pub fn on_complete_boxed (&self, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        self.on_status_boxed(EventStatus::Complete, f)
    }

    #[inline(always)]
    pub fn on_status_boxed (&self, status: EventStatus, f: Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send>) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                let user_data = Box::into_raw(Box::new(f));
                unsafe {
                    if let Err(e) = self.on_status_raw(status, event_listener, user_data.cast()) {
                        let _ = Box::from_raw(user_data);
                        return Err(e);
                    }

                    tri!(clRetainEvent(self.id()));
                }

                return Ok(())
            } else {
                let cb = super::listener::EventCallback { status, cb: super::listener::Callback::Boxed(f) };
                match self.send.send(cb) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorType::InvalidValue.into())
                }
            }
        }
    }
    
    #[inline(always)]
    pub unsafe fn on_submit_raw (&self, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Submitted, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_run_raw (&self, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Running, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_complete_raw (&self, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        Self::on_status_raw(&self, EventStatus::Complete, f, user_data)
    }

    #[inline(always)]
    pub unsafe fn on_status_raw (&self, status: EventStatus, f: unsafe extern "C" fn(event: cl_event, event_command_status: cl_int, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                tri!(opencl_sys::clSetEventCallback(self.id(), status as i32, Some(f), user_data));
                return Ok(())
            } else {
                let cb = super::listener::EventCallback { status, cb: super::listener::Callback::Raw(f, user_data) };
                match self.send.send(cb) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrorType::InvalidValue.into())
                }
            }
        }
    }
}

impl<'a, T, C: Consumer<'a, T>> Deref for Event<T, C> {
    type Target = RawEvent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[docfg(feature = "cl1_1")]
pub type JoinAll<T, C> = Event<Vec<T>, JoinAllConsumer<C>>;

#[docfg(feature = "cl1_1")]
pub struct JoinAllConsumer<C> (Vec<C>);

#[cfg(feature = "cl1_1")]
impl<'a, T, C: Consumer<'a, T>> Consumer<'a, Vec<T>> for JoinAllConsumer<C> {
    #[inline]
    fn consume (self) -> Result<Vec<T>> {
        self.0.into_iter().map(Consumer::consume).try_collect()
    }
}

#[cfg(feature = "cl1_1")]
unsafe extern "C" fn event_listener (event: cl_event, event_command_status: cl_int, user_data: *mut c_void) {
    let user_data : *mut Box<dyn FnOnce(RawEvent, Result<EventStatus>) + Send> = user_data.cast();
    let f = *Box::from_raw(user_data);
    
    let event = RawEvent::from_id_unchecked(event);
    let status = EventStatus::try_from(event_command_status);
    f(event, status)
}