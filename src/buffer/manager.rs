use std::ops::{RangeBounds, Bound};
use std::ptr::addr_of_mut;
use opencl_sys::{cl_event, clEnqueueReadBuffer, CL_FALSE, clEnqueueWriteBuffer};
use crate::context::Context;
use crate::{core::*};
use crate::event::{WaitList};
use super::Buffer;

pub unsafe fn inner_read_to_ptr<T: Copy, C: Context> (src: &Buffer<T, C>, src_range: impl RangeBounds<usize>, dst: *mut T, wait: impl Into<WaitList>) -> Result<cl_event> {
    let (offset, cb) = offset_cb(src, src_range)?;
    let wait : WaitList = wait.into();
    let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

    let mut event = core::ptr::null_mut();
    tri!(clEnqueueReadBuffer(src.ctx.next_queue(), src.inner, CL_FALSE, offset, cb, dst.cast(), num_events_in_wait_list, event_wait_list, &mut event));

    return Ok(event)
}

pub unsafe fn inner_write_from_ptr<T: Copy, C: Context> (dst: &Buffer<T, C>, dst_range: impl RangeBounds<usize>, src: *const T, wait: impl Into<WaitList>) -> Result<cl_event> {
    let (offset, cb) = offset_cb(dst, dst_range)?;
    let (num_events_in_wait_list, event_wait_list) = wait.into().raw_parts();

    let mut event = core::ptr::null_mut();
    tri!(clEnqueueWriteBuffer(dst.ctx.next_queue(), dst.inner, CL_FALSE, offset, cb, src.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

    return Ok(event)
}

pub fn offset_cb<T: Copy, C: Context> (buffer: &Buffer<T, C>, range: impl RangeBounds<usize>) -> Result<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Excluded(x) => *x + 1,
        Bound::Included(x) => *x,
        Bound::Unbounded => 0
    }.checked_mul(core::mem::size_of::<T>()).unwrap();

    let end = match range.end_bound() {
        Bound::Excluded(x) => x.checked_mul(core::mem::size_of::<T>()).unwrap(),
        Bound::Included(x) => (x + 1).checked_mul(core::mem::size_of::<T>()).unwrap(),
        Bound::Unbounded => buffer.byte_size()?
    };

    let len = end - start;
    Ok((start, len))
}

pub fn range_len<T: Copy, C: Context> (buffer: &Buffer<T, C>, range: &impl RangeBounds<usize>) -> Result<usize> {
    let start = match range.start_bound() {
        Bound::Excluded(x) => *x + 1,
        Bound::Included(x) => *x,
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => *x,
        Bound::Included(x) => x + 1,
        Bound::Unbounded => buffer.len()?
    };

    Ok(end - start)
}