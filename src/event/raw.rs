use crate::core::*;
use std::any::Any;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::{resume_unwind};
use std::sync::mpsc::{TryRecvError};
use std::time::{Duration, SystemTime};
use std::{mem::MaybeUninit, ptr::{NonNull}};
use opencl_sys::*;
use blaze_proc::docfg;

use super::ext::NoopEvent;
use super::{EventStatus, ProfilingInfo, CommandType, Event, Consumer};

/// Raw OpenCL event
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawEvent (NonNull<c_void>);

impl RawEvent {
    #[inline(always)]
    pub const unsafe fn from_id_unchecked (inner: cl_event) -> Self {
        Self(NonNull::new_unchecked(inner))
    }

    #[inline(always)]
    pub const unsafe fn from_id (inner: cl_event) -> Option<Self> {
        NonNull::new(inner).map(Self)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_event {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub unsafe fn retain (&self) -> Result<()> {
        tri!(clRetainEvent(self.id()));
        Ok(())
    }

    #[inline(always)]
    pub fn join_by_ref (&self) -> Result<()> {
        let slice = &[self.0.as_ptr()];

        unsafe {
            tri!(clWaitForEvents(1, slice.as_ptr()))
        }

        Ok(())
    }

    /// Blocks the current thread until all the events have completed
    #[inline(always)]
    pub fn join_all_by_ref (v: &[RawEvent]) -> Result<()> {
        let len = u32::try_from(v.len()).unwrap();

        unsafe {
            tri!(clWaitForEvents(len, v.as_ptr().cast()))
        }

        Ok(())
    }
}

impl RawEvent {
    /// Converts the [`RawEvent`] into a [`NoopEvent`].
    #[inline(always)]
    pub fn into_event (self) -> NoopEvent {
        self.into()
    }

    #[inline(always)]
    pub fn join_with_nanos_by_ref (self) -> Result<ProfilingInfo<u64>> {
        self.join_by_ref()?;
        self.profiling_nanos()
    }

    #[inline(always)]
    pub fn join_with_time_by_ref (self) -> Result<ProfilingInfo<SystemTime>> {
        self.join_by_ref()?;
        self.profiling_time()
    }

    #[inline(always)]
    pub fn join_with_duration_by_ref (self) -> Result<Duration> {
        self.join_by_ref()?;
        self.duration()
    }

    /// Blocks the current thread util the event has completed, returning `data` if it completed correctly, and panicking otherwise.
    #[inline(always)]
    pub fn join_unwrap_by_ref (self) {
        self.join_by_ref().unwrap()
    }

    /// Returns the event's type
    #[inline(always)]
    pub fn ty (&self) -> Result<CommandType> {
        self.get_info(CL_EVENT_COMMAND_TYPE)
    }

    /// Returns the event's current status
    #[inline(always)]
    pub fn status (&self) -> Result<EventStatus> {
        let int : i32 = self.get_info(CL_EVENT_COMMAND_EXECUTION_STATUS)?;
        EventStatus::try_from(int)
    }

    /// Returns the event's underlying command queue
    #[inline(always)]
    pub fn command_queue (&self) -> Result<Option<RawCommandQueue>> {
        match self.get_info(CL_EVENT_COMMAND_QUEUE) {
            Ok(x) => unsafe { Ok(RawCommandQueue::from_id(x)) },
            Err(e) => Err(e)
        }
    }

    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(opencl_sys::CL_EVENT_REFERENCE_COUNT)
    }

    /// Return the context associated with event.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn raw_context (&self) -> Result<crate::prelude::RawContext> {
        let ctx = self.get_info::<cl_context>(CL_EVENT_CONTEXT)?;
        unsafe { 
            tri!(clRetainContext(ctx));
            // SAFETY: Context checked to be valid by `clRetainContext`.
            Ok(crate::prelude::RawContext::from_id_unchecked(ctx))
        }
    }

    /// Returns this event's profiling info in `u64` nanoseconds.
    #[inline(always)]
    pub fn profiling_nanos (&self) -> Result<ProfilingInfo<u64>> {
        ProfilingInfo::<u64>::new(self)
    }

    /// Returns this event's profiling info in [`SystemTime`].
    #[inline(always)]
    pub fn profiling_time (&self) -> Result<ProfilingInfo<SystemTime>> {
        ProfilingInfo::<SystemTime>::new(self)
    }

    /// Returns the time elapsed between the event's start and end.
    #[inline(always)]
    pub fn duration (&self) -> Result<Duration> {
        let nanos = self.profiling_nanos()?;
        Ok(nanos.duration())
    }

    /// Returns `true` if the status of the event is [`EventStatus::Queued`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn is_queued (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::is_queued)
    }

    /// Returns `true` if the status of the event is [`EventStatus::Submitted`], [`EventStatus::Running`], [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn has_submited (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_submitted)
    }

    /// Returns `true` if the status of the event is [`EventStatus::Running`], [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn has_started_running (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_started_running)
    }
    
    /// Returns `true` if the status of the event is [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    pub fn has_completed (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_completed)
    }
    
    #[inline(always)]
    pub fn get_info<T: Copy> (&self, id: cl_event_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetEventInfo(self.id(), id, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

#[docfg(feature = "cl1_1")]
impl RawEvent {
    #[inline(always)]
    pub fn on_submit<T: 'static + Send> (&self, f: impl 'static + Send + FnOnce(RawEvent, Result<EventStatus>) -> T) -> Result<CallbackHandle<T>> { 
        self.on_status(EventStatus::Submitted, f)
    }

    #[inline(always)]
    pub fn on_run<T: 'static + Send> (&self, f: impl 'static + Send + FnOnce(RawEvent, Result<EventStatus>) -> T) -> Result<CallbackHandle<T>> { 
        self.on_status(EventStatus::Running, f)
    }

    #[inline(always)]
    pub fn on_complete<T: 'static + Send> (&self, f: impl 'static + Send + FnOnce(RawEvent, Result<EventStatus>) -> T) -> Result<CallbackHandle<T>> { 
        self.on_status(EventStatus::Complete, f)
    }

    /// Adds a callback function that will be executed when the event reaches the specified status.
    /// 
    /// Unlike [`on_status_silent`], this method returns a [`CallbackHandle`] that allows to wait for the callcack to execute and getting its result.
    pub fn on_status<T: 'static + Send> (&self, status: EventStatus, f: impl 'static + Send + FnOnce(RawEvent, Result<EventStatus>) -> T) -> Result<CallbackHandle<T>> {        
        let (send, recv) = std::sync::mpsc::sync_channel::<_>(1);
        let data = std::sync::Arc::new(CallbackHandleData {
            #[cfg(feature = "cl1_1")]
            flag: once_cell::sync::OnceCell::new(),
            #[cfg(feature = "future")]
            waker: futures::task::AtomicWaker::new()
        });
        
        let my_data = data.clone();
        self.on_status_silent(status, move |evt, status| {
            let f = std::panic::AssertUnwindSafe(|| f(evt, status.clone()));
            match send.send(std::panic::catch_unwind(f)) {
                Ok(_) => {
                    #[cfg(feature = "cl1_1")]
                    if let Some(flag) = my_data.flag.get_or_init(|| None) {
                        flag.try_mark(status.err().map(|x| x.ty)).unwrap();
                    }
                    #[cfg(feature = "futures")]
                    my_data.waker.wake();
                },
                Err(_) => {}
            }
        })?;

        return Ok(CallbackHandle { recv, data, phtm: PhantomData })
    }

    /// Adds a callback function that will be executed when the event is submitted.
    #[inline(always)]
    pub fn on_submit_silent (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status_silent(EventStatus::Submitted, f)
    }

    /// Adds a callback function that will be executed when the event starts running.
    #[inline(always)]
    pub fn on_run_silent (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status_silent(EventStatus::Running, f)
    }

    /// Adds a callback function that will be executed when the event completes.
    #[inline(always)]
    pub fn on_complete_silent (&self, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        self.on_status_silent(EventStatus::Complete, f)
    }

    /// Registers a user callback function for a specific command execution status.\
    /// The registered callback function will be called when the execution status of command associated with event changes to an execution status equal to or past the status specified by `status`.\
    /// Each call to [`Event::on_status`] registers the specified user callback function on a callback stack associated with event. The order in which the registered user callback functions are called is undefined.\
    /// All callbacks registered for an event object must be called before the event object is destroyed. Callbacks should return promptly.\
    /// Behavior is undefined when calling expensive system routines, OpenCL APIs to create contexts or command-queues, or blocking OpenCL APIs in an event callback. Rather than calling a blocking OpenCL API in an event callback, applications may call a non-blocking OpenCL API, then register a completion callback for the non-blocking OpenCL API with the remainder of the work.\
    /// Because commands in a command-queue are not required to begin execution until the command-queue is flushed, callbacks that enqueue commands on a command-queue should either call [`RawCommandQueue::flush`] on the queue before returning, or arrange for the command-queue to be flushed later.
    #[inline(always)]
    pub fn on_status_silent (&self, status: EventStatus, f: impl 'static + FnOnce(RawEvent, Result<EventStatus>) + Send) -> Result<()> {
        let f: Box<Box<dyn 'static + FnOnce(RawEvent, Result<EventStatus>) + Send>> = Box::new(Box::new(f));
        let user_data = Box::into_raw(f);

        unsafe {
            if let Err(e) = self.on_status_raw(status, event_listener, user_data.cast()) {
                let _ = Box::from_raw(user_data); // drop user data
                return Err(e);
            }

            tri!(clRetainEvent(self.id()));
            return Ok(())
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
        tri!(opencl_sys::clSetEventCallback(self.id(), status as i32, Some(f), user_data));
        return Ok(())
    }
}

impl Into<NoopEvent> for RawEvent {
    #[inline(always)]
    fn into(self) -> NoopEvent {
        Event::new_noop(self)
    }
}

impl Clone for RawEvent {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainEvent(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for RawEvent {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseEvent(self.id()))
        }
    }
}

unsafe impl Send for RawEvent {}
unsafe impl Sync for RawEvent {}

pub(crate) unsafe extern "C" fn event_listener (event: cl_event, event_command_status: cl_int, user_data: *mut c_void) {
    let f = *Box::from_raw(user_data.cast::<Box<dyn 'static + Send + FnOnce(RawEvent, Result<EventStatus>)>>());
    let event = RawEvent::from_id_unchecked(event);
    let status = EventStatus::try_from(event_command_status);
    f(event, status)
}

pub type CallbackHandle<T> = ScopedCallbackHandle<'static, T>;
pub type CallbackConsumer<T> = ScopedCallbackConsumer<'static, T>;

pub(crate) struct CallbackHandleData {
    #[cfg(feature = "cl1_1")]
    pub(crate) flag: once_cell::sync::OnceCell<Option<super::FlagEvent>>,
    #[cfg(feature = "futures")]
    pub(crate) waker: futures::task::AtomicWaker
}

pub struct ScopedCallbackHandle<'a, T> {
    pub(crate) recv: std::sync::mpsc::Receiver<std::thread::Result<T>>,
    pub(crate) data: std::sync::Arc<CallbackHandleData>,
    pub(crate) phtm: PhantomData<&'a mut &'a ()>
}

impl<'a, T> ScopedCallbackHandle<'a, T> {
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn into_event (self) -> Result<Event<ScopedCallbackConsumer<'a, T>>> {
        self.into_event_in(crate::context::Global)
    }

    #[docfg(feature = "cl1_1")]
    pub fn into_event_in<C: crate::prelude::Context> (self, ctx: C) -> Result<Event<ScopedCallbackConsumer<'a, T>>> {
        let flag = super::FlagEvent::new_in(ctx.as_raw())?;
        let sub = flag.subscribe();

        match self.data.flag.try_insert(Some(flag)) {
            // Flag registered
            Ok(_) => {},

            // Callback has already completed
            Err((_, flag)) => unsafe {
                flag.unwrap_unchecked().try_mark(None)?;
            }
        }

        return Ok(Event::new(sub, ScopedCallbackConsumer(self)));
    }

    #[inline]
    pub fn join (self) -> std::thread::Result<T> {
        return match self.recv.recv() {
            Ok(x) => x,
            Err(_) => panic!("Handle already joined")
        }
    }

    #[inline]
    pub fn join_unwrap (self) -> T {
        return match self.recv.recv() {
            Ok(Ok(x)) => x,
            Ok(Err(e)) => resume_unwind(e),
            Err(_) => panic!("Handle already joined")
        }
    }

    #[inline]
    pub fn try_join (self) -> ::core::result::Result<std::thread::Result<T>, Self> {
        return match self.recv.try_recv() {
            Ok(x) => Ok(x),
            Err(TryRecvError::Empty) => Err(self),
            Err(TryRecvError::Disconnected) => panic!("Handle already joined")
        }
    }

    #[inline]
    pub fn try_join_unwrap (self) -> ::core::result::Result<T, Self> {
        return match self.recv.try_recv() {
            Ok(Ok(x)) => Ok(x),
            Ok(Err(e)) => resume_unwind(e),
            Err(TryRecvError::Empty) => Err(self),
            Err(TryRecvError::Disconnected) => panic!("Handle already joined")
        }
    }
}

#[repr(transparent)]
pub struct ScopedCallbackConsumer<'a, T> (ScopedCallbackHandle<'a, T>);

impl<'a, T> Consumer for ScopedCallbackConsumer<'a, T> {
    type Output = ::core::result::Result<T, Box<dyn 'static + Any + Send>>;

    #[inline]
    unsafe fn consume (self) -> Result<Self::Output> {
        // Optimistic lock
        loop {
            match self.0.recv.try_recv() {
                Ok(x) => return Ok(x),
                Err(TryRecvError::Disconnected) => todo!(),
                Err(TryRecvError::Empty) => core::hint::spin_loop()
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        use futures::future::*;
        use std::task::*;

        #[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
        impl<T> Future for ScopedCallbackHandle<'_, T> {
            type Output = std::thread::Result<T>;

            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                self.data.waker.register(cx.waker());
                return match self.recv.try_recv() {
                    Ok(x) => Poll::Ready(x),
                    Err(TryRecvError::Empty) => Poll::Pending,
                    Err(TryRecvError::Disconnected) => panic!("Handle already joined")
                }
            }
        }
    }
}

pub(crate) mod sealed {
    use thin_trait_object::*;
    use crate::prelude::*;
    use crate::event::EventStatus;
    use super::RawEvent;
    
    #[thin_trait_object]
    pub trait SilentCallback {
        unsafe fn call (&mut self, args: (RawEvent, Result<EventStatus>,));
    }

    impl<F: Send + FnOnce(RawEvent, Result<EventStatus>)> SilentCallback for F {
        #[inline(always)]
        unsafe fn call (&mut self, args: (RawEvent, Result<EventStatus>,)) {
            core::ptr::read(self).call_once(args)
        }
    }
}