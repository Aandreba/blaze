use std::{ops::Deref, ptr::{addr_of_mut, NonNull}};
use opencl_sys::*;
use crate::{prelude::{Buffer, Context, Result, WaitList, Error, RawEvent}, buffer::{IntoRange, BufferRange}};

pub struct MapBuffer<T, S> {
    evt: RawEvent,
    ptr: NonNull<[T]>,
    src: S
}

impl<T: Copy, S: Deref<Target = Buffer<T, C>>, C: Context> MapBuffer<T, S> {
    pub fn new (src: S, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<Self> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        let BufferRange { offset, cb } = range.into_range::<T>(&src)?;

        let mut err = 0;
        let mut evt = core::ptr::null_mut();

        unsafe {
            let ptr = clEnqueueMapBuffer(src.ctx.next_queue().id(), src.id(), CL_FALSE, CL_MAP_READ, offset, cb, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt), addr_of_mut!(err));
    
            if err != 0 {
                return Err(Error::from(err))
            }
    
            let evt = RawEvent::from_id(evt).unwrap();
            let ptr = {
                let slice = core::slice::from_raw_parts_mut(ptr as *mut T, cb / core::mem::size_of::<T>());
                NonNull::new(slice)
            }.ok_or_else(|| Error::from_type(crate::prelude::ErrorType::InvalidValue))?;
    
            Ok(Self { evt, src, ptr })
        }
    }
}