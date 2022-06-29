use std::ops::{RangeBounds, Bound};
use std::ptr::addr_of_mut;
use opencl_sys::{cl_event, clEnqueueReadBuffer, CL_FALSE, clEnqueueWriteBuffer};
use crate::{core::*};
use crate::event::{WaitList, RawEvent};
use super::{RawBuffer};

#[derive(Clone)]
pub enum AccessManager {
    None,
    Reading (Vec<RawEvent>),
    Writing (RawEvent)
}

impl AccessManager {
    #[inline]
    pub fn extend_list (&self, wait: &mut WaitList) {
        match self {
            Self::Reading(x) => wait.extend(x.into_iter().cloned()),
            Self::Writing(x) => wait.push(x.clone()),
            Self::None => {},
        }
    }

    #[inline]
    pub fn read (&mut self, evt: RawEvent) -> WaitList {
        match self {
            Self::None => {
                *self = Self::Reading(vec![evt]);
                WaitList::EMPTY
            },

            Self::Reading(x) => {
                x.push(evt);
                WaitList::EMPTY
            },

            Self::Writing(x) => {
                let wait = WaitList::from_event(x.clone());
                *self = Self::Reading(vec![evt]);
                wait
            }
        }
    }

    #[inline]
    pub fn write (&mut self, evt: RawEvent) -> WaitList {
        match self {
            Self::None => {
                *self = Self::Writing(evt);
                WaitList::EMPTY
            },

            Self::Reading(x) => {
                let wait = WaitList::new(core::mem::take(x));
                *self = Self::Writing(evt);
                wait
            },

            Self::Writing(x) => WaitList::from_event(core::mem::replace(x, evt))
        }
    }
}

pub unsafe fn inner_read_to_ptr<T: Copy> (src: &RawBuffer, src_range: impl RangeBounds<usize>, dst: *mut T, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<cl_event> {
    let (offset, cb) = offset_cb(&src, core::mem::size_of::<T>(), src_range)?;
    let wait : WaitList = wait.into();
    let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

    let mut event = core::ptr::null_mut();
    tri!(clEnqueueReadBuffer(queue.id(), src.id(), CL_FALSE, offset, cb, dst.cast(), num_events_in_wait_list, event_wait_list, &mut event));

    return Ok(event)
}

pub unsafe fn inner_write_from_ptr<T: Copy> (dst: &mut RawBuffer, dst_range: impl RangeBounds<usize>, src: *const T, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<cl_event> {
    let (offset, cb) = offset_cb(&dst, core::mem::size_of::<T>(), dst_range)?;
    let wait : WaitList = wait.into();
    let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

    let mut event = core::ptr::null_mut();
    tri!(clEnqueueWriteBuffer(queue.id(), dst.id(), CL_FALSE, offset, cb, src.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

    return Ok(event)
}

#[inline]
pub fn offset_cb (buffer: &RawBuffer, size: usize, range: impl RangeBounds<usize>) -> Result<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Excluded(x) => x.checked_add(1).and_then(|x| x.checked_mul(size)).unwrap(),
        Bound::Included(x) => x.checked_mul(size).unwrap(),
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => x.checked_mul(size).unwrap(),
        Bound::Included(x) => x.checked_add(1).and_then(|x| x.checked_mul(size)).unwrap(),
        Bound::Unbounded => buffer.size()?
    };

    let len = end - start;
    Ok((start, len))
}

#[inline]
pub fn range_len (buffer: &RawBuffer, len: usize, range: &impl RangeBounds<usize>) -> usize {
    let start = match range.start_bound() {
        Bound::Excluded(x) => *x + 1,
        Bound::Included(x) => *x,
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => *x,
        Bound::Included(x) => x + 1,
        Bound::Unbounded => len
    };

    end - start
}