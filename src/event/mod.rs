use std::{mem::ManuallyDrop, time::{SystemTime, Duration}, ops::Deref, alloc::Allocator, panic::AssertUnwindSafe};
use opencl_sys::{CL_COMMAND_NDRANGE_KERNEL, CL_COMMAND_TASK, CL_COMMAND_NATIVE_KERNEL, CL_COMMAND_READ_BUFFER, CL_COMMAND_WRITE_BUFFER, CL_COMMAND_COPY_BUFFER, CL_COMMAND_READ_IMAGE, CL_COMMAND_WRITE_IMAGE, CL_COMMAND_COPY_IMAGE, CL_COMMAND_COPY_IMAGE_TO_BUFFER, CL_COMMAND_COPY_BUFFER_TO_IMAGE, CL_COMMAND_MAP_BUFFER, CL_COMMAND_MAP_IMAGE, CL_COMMAND_UNMAP_MEM_OBJECT, CL_COMMAND_MARKER, CL_COMMAND_ACQUIRE_GL_OBJECTS, CL_COMMAND_RELEASE_GL_OBJECTS, CL_EVENT_COMMAND_TYPE, CL_EVENT_COMMAND_EXECUTION_STATUS, CL_EVENT_COMMAND_QUEUE, cl_event};
use blaze_proc::docfg;
use crate::{core::*, prelude::RawContext};

flat_mod!(status, raw, various, info);

#[cfg(feature = "cl1_1")]
flat_mod!(flag, join);

#[cfg(feature = "futures")]
flat_mod!(wait);

#[cfg(all(feature = "cl1_1", feature = "futures"))]
flat_mod!(future);

/// An complex OpenCL event, with a syntax simillar to Rust's [`Future`](std::future::Future).\
/// [`Event`] is designed to be able to safely return a value after the underlying [`RawEvent`] has completed
pub trait Event {
    type Output;

    /// Returns a reference to the underlying [`RawEvent`]
    fn as_raw (&self) -> &RawEvent;

    /// Consumes the event, returning the data associated with it.
    fn consume (self, err: Option<Error>) -> Result<Self::Output>;

    /// Returns the event whose the logic of this one depends on. This event is the one that will be awaited by [`Event::wait`] and [`Event::wait_async`].\
    /// By default, this funtion returns the same as [`Event::as_raw`]
    #[inline(always)]
    fn parent_event (&self) -> &RawEvent {
        self.as_raw()
    }

    /// Returns the underlying [`RawEvent`]
    #[inline(always)]
    fn to_raw (&self) -> RawEvent {
        self.as_raw().clone()
    }

    /// Blocks the current thread util the event has completed, returning `Ok(data)` if it completed correctly, and `Err(e)` otherwise.
    #[inline(always)]
    fn wait (self) -> Result<Self::Output> where Self: Sized {
        let err = self.wait_by_ref().err();
        self.consume(err)
    }

    #[inline(always)]
    fn wait_with_nanos (self) -> Result<(Self::Output, ProfilingInfo<u64>)> where Self: Sized {
        let err = self.wait_by_ref().err();
        let profile = self.profiling_nanos()?;
        self.consume(err).map(|x| (x, profile))
    }

    #[inline(always)]
    fn wait_with_time (self) -> Result<(Self::Output, ProfilingInfo<SystemTime>)> where Self: Sized {
        let err = self.wait_by_ref().err();
        let profile = self.profiling_time()?;
        self.consume(err).map(|x| (x, profile))
    }

    #[inline(always)]
    fn wait_with_duration (self) -> Result<(Self::Output, Duration)> where Self: Sized {
        let err = self.wait_by_ref().err();
        let profile = self.duration()?;
        self.consume(err).map(|x| (x, profile))
    }

    /// Blocks the current thread util the event has completed, returning `data` if it completed correctly, and panicking otherwise.
    #[inline(always)]
    fn wait_unwrap (self) -> Self::Output where Self: Sized {
        self.wait().unwrap()
    }

    /// Returns a future that waits for the event to complete without blocking.
    #[inline(always)]
    #[docfg(feature = "futures")]
    fn wait_async (self) -> Result<crate::event::EventWait<Self>> where Self: Sized {
        crate::event::EventWait::new(self)
    }

    /// Blocks the current thread until the event has completed, without returning it's underlying data.
    #[inline(always)]
    fn wait_by_ref (&self) -> Result<()> {
        self.parent_event().wait_by_ref()
    }

    /// Returns the event's type
    #[inline(always)]
    fn ty (&self) -> Result<CommandType> {
        self.as_raw().get_info(CL_EVENT_COMMAND_TYPE)
    }

    /// Returns the event's current status
    #[inline(always)]
    fn status (&self) -> Result<EventStatus> {
        let int : i32 = self.as_raw().get_info(CL_EVENT_COMMAND_EXECUTION_STATUS)?;
        EventStatus::try_from(int)
    }

    /// Returns the event's underlying command queue
    #[inline(always)]
    fn command_queue (&self) -> Result<Option<RawCommandQueue>> {
        self.as_raw().get_info(CL_EVENT_COMMAND_QUEUE).map(RawCommandQueue::from_id)
    }

    /// Return the context associated with event.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    fn raw_context (&self) -> Result<crate::prelude::RawContext> {
        self.as_raw().get_info(opencl_sys::CL_EVENT_CONTEXT)
    }

    /// Returns this event's profiling info in `u64` nanoseconds.
    #[inline(always)]
    fn profiling_nanos (&self) -> Result<ProfilingInfo<u64>> {
        ProfilingInfo::<u64>::new(self.as_raw())
    }

    /// Returns this event's profiling info in [`SystemTime`].
    #[inline(always)]
    fn profiling_time (&self) -> Result<ProfilingInfo<SystemTime>> {
        ProfilingInfo::<SystemTime>::new(self.as_raw())
    }

    /// Returns the time elapsed between the event's start and end.
    #[inline(always)]
    fn duration (&self) -> Result<Duration> {
        let nanos = self.profiling_nanos()?;
        Ok(nanos.duration())
    }

    /// Returns `true` if the status of the event is [`EventStatus::Complete`] or an error, `false` otherwise.
    #[inline(always)]
    fn has_completed (&self) -> bool {
        self.status().as_ref().map_or(true, EventStatus::has_completed)
    }
}

impl<T: Event, A: Allocator> Event for Box<T, A> {
    type Output = T::Output;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.deref().as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        Box::into_inner(self).consume(err)
    }
}

impl<T: Event> Event for AssertUnwindSafe<T> {
    type Output = T::Output;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.deref().as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        self.0.consume(err)
    }
}

pub trait EventExt: Sized + Event {
    /// Executes the specified function after the parent event has completed. 
    #[inline]
    fn map<T, F: FnOnce(Self::Output) -> T> (self, f: F) -> Result<Map<Self, F>> {
        Ok(Map {
            parent: self,
            f
        })
    }

    /// Executes the specified function after the parent event has completed. 
    #[docfg(feature = "cl1_1")]
    #[inline]
    fn then<T, F: FnOnce(Self::Output) -> T> (self, f: F) -> Result<Then<Self, F>> {
        let ctx = self.raw_context()?;
        let flag = FlagEvent::new_in(&ctx)?;

        Ok(Then {
            parent: self,
            flag,
            f
        })
    }

    #[inline(always)]
    fn inspect<F: FnOnce(&Self::Output)> (self, f: F) -> Inspect<Self, F> {
        Inspect {
            parent: self,
            f
        }
    }

    /// Returns an event that completes when all the events inside `iter` have completed.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    fn join<I: IntoIterator<Item = Self>> (iter: I) -> Result<EventJoin<Self>> where Self: 'static + Send, Self::Output: Unpin + Send + Sync, I::IntoIter: ExactSizeIterator {
        EventJoin::new(iter)
    }

    /// Blocks the current thread until all the events inside `iter` have completed.\
    /// Unlike [`join`](EventExt::join) and [`join_ordered`](EventExt::join_ordered), this method does not require it's types to implement `'static`, [`Send`], [`Sync`] or [`Unpin`], nor does it require `iter` to be [`ExactSizeIterator`]. \
    /// The return vector maintains the same order as `iter`.
    #[inline]
    fn join_blocking<I: IntoIterator<Item = Self>> (iter: I) -> Result<Vec<Self::Output>> {
        let (raw, iter) : (Vec<_>, Vec<_>) = iter.into_iter()
            .map(|x| (x.to_raw(), x))
            .unzip();

        let err = RawEvent::wait_all(&raw).err();
        let mut result = Vec::with_capacity(iter.len());
        
        for evt in iter {
            let v = evt.consume(err.clone())?;
            result.push(v);
        }
        
        Ok(result)
    }

    /// Returns an event that completes when all the events inside `iter` have completed, ensuring that the order of the outputs is the same as the inputs 
    /// (first output is the result of the first event in the iterator, second output is from second event, etc.)
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    fn join_ordered<I: IntoIterator<Item = Self>> (iter: I) -> Result<EventJoinOrdered<Self>> where Self: 'static + Send, Self::Output: Unpin + Send + Sync, I::IntoIter: ExactSizeIterator {
        EventJoinOrdered::new(iter)
    }

    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    fn join_in<I: IntoIterator<Item = Self>> (ctx: &RawContext, iter: I) -> Result<EventJoin<Self>> where Self: 'static + Send, Self::Output: Unpin + Send + Sync, I::IntoIter: ExactSizeIterator {
        EventJoin::new_in(ctx, iter)
    }

    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    fn join_ordered_in<I: IntoIterator<Item = Self>> (ctx: &RawContext, iter: I) -> Result<EventJoinOrdered<Self>> where Self: 'static + Send, Self::Output: Unpin + Send + Sync, I::IntoIter: ExactSizeIterator {
        EventJoinOrdered::new_in(ctx, iter)
    }
}

impl<T: Event> EventExt for T {}

/// A list of events to be awaited
#[derive(Clone)]
#[repr(transparent)]
pub struct WaitList (pub(crate) Option<Vec<RawEvent>>);

impl WaitList {
    /// An empty wait list
    pub const EMPTY : Self = WaitList(None);

    /// Creates a new wait list
    #[inline(always)]
    pub fn new (wait: Vec<RawEvent>) -> Self {
        if wait.len() == 0 {
            return Self::EMPTY;
        }

        Self(Some(wait))
    }

    /// Creates a new wait list from the specified array
    #[inline(always)]
    pub fn from_array<const N: usize> (wait: [RawEvent;N]) -> Self {
        if N == 0 {
            return Self::EMPTY;
        }
        
        Self(Some(wait.to_vec()))
    }

    /// Creates a new wait list from a single event
    #[inline(always)]
    pub fn from_event (wait: RawEvent) -> Self {
        Self(Some(vec![wait]))
    }

    /// Adds a new event to the list
    #[inline(always)]
    pub fn push (&mut self, evt: RawEvent) {
        match self.0 {
            Some(ref mut x) => x.push(evt),
            None => self.0 = Some(vec![evt])
        }
    }

    /// Returns the parts of the wait list necessary to be passed to a raw OpenCL function
    #[inline(always)]
    pub fn raw_parts (&self) -> (u32, *const cl_event) {
        match self.0 {
            Some(ref x) => {
                (u32::try_from(x.len()).unwrap(), x.as_ptr().cast())
            },
            
            None => (0, core::ptr::null())
        }
    }

    /// Extends the current wait list with the events of `other`
    pub fn extend_self (&mut self, wait: WaitList) {
        if let Some(y) = wait.0 {
            match self.0 {
                Some(ref mut x) => {
                    let y = ManuallyDrop::new(y);
                    let (y_len, y_ptr) = (y.len(), y.as_ptr());

                    unsafe {
                        x.reserve(y_len);
                        core::ptr::copy_nonoverlapping(y_ptr, x.as_mut_ptr().add(x.len()), y_len);
                        x.set_len(x.len() + y_len)
                    }
                }

                None => self.0 = Some(y)
            }
        }
    }

    #[inline(always)]
    pub fn wait_all (&self) -> Result<()> {
        match self.0 {
            Some(ref x) => RawEvent::wait_all(&x),
            None => Ok(())
        }
    }
}

impl Extend<RawEvent> for WaitList {
    fn extend<T: IntoIterator<Item = RawEvent>>(&mut self, iter: T) {
        let iter = iter.into_iter();

        match self.0 {
            Some(ref mut x) => x.extend(iter),
            None => *self = Self::new(iter.collect::<Vec<_>>())
        }
    }

    #[inline(always)]
    fn extend_one(&mut self, item: RawEvent) {
        self.push(item)
    }
}

impl<T: Event> From<&T> for WaitList {
    #[inline(always)]
    fn from(x: &T) -> Self {
        Self::from_event(T::as_raw(x).clone())
    }
}

impl From<Vec<RawEvent>> for WaitList {
    #[inline(always)]
    fn from(x: Vec<RawEvent>) -> Self {
        Self::new(x)
    }
}

impl<const N: usize> From<[RawEvent; N]> for WaitList {
    #[inline(always)]
    fn from(x: [RawEvent; N]) -> Self {
        Self::from_array(x)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum CommandType {
    NdRangeKernel = CL_COMMAND_NDRANGE_KERNEL,
    Task = CL_COMMAND_TASK,
    NativeKernel = CL_COMMAND_NATIVE_KERNEL,
    ReadBuffer = CL_COMMAND_READ_BUFFER,
    WriteBuffer = CL_COMMAND_WRITE_BUFFER,
    CopyBuffer = CL_COMMAND_COPY_BUFFER,
    ReadImage = CL_COMMAND_READ_IMAGE,
    WriteImage = CL_COMMAND_WRITE_IMAGE,
    CopyImage = CL_COMMAND_COPY_IMAGE,
    CopyImageToBuffer = CL_COMMAND_COPY_IMAGE_TO_BUFFER,
    CopyBufferToImage = CL_COMMAND_COPY_BUFFER_TO_IMAGE,
    MapBuffer = CL_COMMAND_MAP_BUFFER,
    MapImage = CL_COMMAND_MAP_IMAGE,
    UnmapMemObject = CL_COMMAND_UNMAP_MEM_OBJECT,
    Marker = CL_COMMAND_MARKER,
    AcquireGLObjects = CL_COMMAND_ACQUIRE_GL_OBJECTS,
    ReleaseGLObjects = CL_COMMAND_RELEASE_GL_OBJECTS
}