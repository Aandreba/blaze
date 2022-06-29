use std::ffi::c_void;
use opencl_sys::{cl_event, cl_int, clReleaseMemObject};

flat_mod!(read, write);

pub(in crate::buffer) unsafe extern "C" fn drop_buffer (_event: cl_event, _event_command_status: cl_int, user_data: *mut c_void) {
    tri_panic!(clReleaseMemObject(user_data))
}