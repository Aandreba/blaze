use std::{mem::MaybeUninit, ffi::c_void, ptr::{addr_of_mut, NonNull}};
use opencl_sys::{cl_kernel, cl_kernel_info, clRetainProgram, CL_KERNEL_PROGRAM, CL_KERNEL_CONTEXT, CL_KERNEL_REFERENCE_COUNT, CL_KERNEL_NUM_ARGS, CL_KERNEL_FUNCTION_NAME, CL_KERNEL_ARG_ADDRESS_GLOBAL, CL_KERNEL_ARG_ADDRESS_LOCAL, CL_KERNEL_ARG_ADDRESS_CONSTANT, CL_KERNEL_ARG_ADDRESS_PRIVATE, cl_kernel_arg_type_qualifier, CL_KERNEL_ARG_TYPE_CONST, CL_KERNEL_ARG_TYPE_RESTRICT, CL_KERNEL_ARG_TYPE_VOLATILE, clGetKernelInfo, clSetKernelArg, clEnqueueNDRangeKernel, clRetainKernel, clReleaseKernel};
use rscl_proc::docfg;
use crate::{core::*, context::{RawContext, Context, Global}, event::{RawEvent, WaitList}};

#[repr(transparent)]
pub struct Kernel (NonNull<c_void>);

impl Kernel {
    #[inline(always)]
    pub const fn id (&self) -> cl_kernel {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_kernel) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    #[inline(always)]
    pub const fn from_id (id: cl_kernel) -> Option<Self> {
        NonNull::new(id).map(Self)
    }

    #[inline(always)]
    pub fn set_argument<T: ?Sized> (&mut self, idx: u32, v: &T) -> Result<()> {
        let ptr = v as *const _ as *const _;

        unsafe {
            tri!(clSetKernelArg(self.id(), idx, core::mem::size_of_val(v), ptr))
        }

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn set_ptr_argument (&mut self, idx: u32, size: usize, ptr: *const c_void) -> Result<()> {
        tri!(clSetKernelArg(self.id(), idx, size, ptr));
        Ok(())
    }

    #[inline(always)]
    pub fn allocate_argument (&mut self, idx: u32, size: usize) -> Result<()> {
        unsafe { self.set_ptr_argument(idx, size, core::ptr::null()) }
    }

    #[docfg(feature = "svm")]
    pub unsafe fn set_svm_argument (&mut self, idx: u32, v: &impl crate::svm::SvmPointer) -> Result<()> {
        tri!(opencl_sys::clSetKernelArgSVMPointer(self.id(), idx, v.as_ptr().cast()));
        Ok(())
    }

    #[inline(always)]
    pub fn enqueue<const N: usize> (&mut self, global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        self.enqueue_with_queue(Global.next_queue(), global_work_dims, local_work_dims, wait)
    }

    #[inline(always)]
    pub fn enqueue_with_context<C: Context, const N: usize> (&mut self, ctx: &C, global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        self.enqueue_with_queue(ctx.next_queue(), global_work_dims, local_work_dims, wait)
    }

    pub fn enqueue_with_queue<const N: usize> (&mut self, queue: &CommandQueue, global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let work_dim = u32::try_from(N).expect("Integer overflow");
        let local_work_dims = local_work_dims.into();
        let local_work_dims = match local_work_dims {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        unsafe {
            tri!(clEnqueueNDRangeKernel(queue.id(), self.id(), work_dim, core::ptr::null(), global_work_dims.as_ptr(), local_work_dims, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)))
        }

        Ok(RawEvent::from_id(event).unwrap())
    }

    /// Return the kernel function name.
    #[inline(always)]
    pub fn name (&self) -> Result<String> {
        self.get_info_string(CL_KERNEL_FUNCTION_NAME)
    }

    /// Return the number of arguments to _kernel_.
    #[inline(always)]
    pub fn num_args (&self) -> Result<u32> {
        self.get_info(CL_KERNEL_NUM_ARGS)
    }

    /// Return the _kernel_ reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_KERNEL_REFERENCE_COUNT)
    }

    /// Return the context associated with _kernel_.
    #[inline(always)]
    pub fn raw_context (&self) -> Result<RawContext> {
        self.get_info(CL_KERNEL_CONTEXT)
    }

    /// Return the program object associated with _kernel_.
    #[inline(always)]
    pub fn program (&self) -> Result<Program> {
        let prog : Program = self.get_info(CL_KERNEL_PROGRAM)?;
        unsafe { tri_panic!(clRetainProgram(prog.id())); }
        Ok(prog)
    }

    #[inline(always)]
    pub unsafe fn clone (&self) -> Self {
        tri_panic!(clRetainKernel(self.id()));
        Self(self.0)
    }

    #[inline]
    fn get_info_string (&self, ty: cl_kernel_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetKernelInfo(self.id(), ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetKernelInfo(self.id(), ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));

            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_kernel_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetKernelInfo(self.id(), ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }
}

impl Drop for Kernel {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseKernel(self.id()))
        }
    }
}

unsafe impl Send for Kernel {}
unsafe impl Sync for Kernel {}

#[cfg(feature = "cl1_2")]
use {crate::buffer::flags::MemAccess, opencl_sys::{CL_KERNEL_ARG_NAME, CL_KERNEL_ARG_ADDRESS_QUALIFIER, CL_KERNEL_ARG_ACCESS_QUALIFIER, CL_KERNEL_ARG_TYPE_QUALIFIER, CL_KERNEL_ARG_TYPE_NAME, cl_kernel_arg_info, clGetKernelArgInfo}};

#[docfg(feature = "cl1_2")]
impl Kernel {
    /// Returns the address qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_address_qualifier (&self, idx: u32) -> Result<AddrQualifier> {
        self.get_arg_info(CL_KERNEL_ARG_ADDRESS_QUALIFIER, idx)
    }

    /// Returns the access qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_access_qualifier (&self, idx: u32) -> Result<MemAccess> {
        let flags = self.get_arg_info::<opencl_sys::cl_kernel_arg_access_qualifier>(CL_KERNEL_ARG_ACCESS_QUALIFIER, idx)?;
        let v = match flags {
            opencl_sys::CL_KERNEL_ARG_ACCESS_READ_ONLY => MemAccess::READ_ONLY,
            opencl_sys::CL_KERNEL_ARG_ACCESS_WRITE_ONLY => MemAccess::WRITE_ONLY,
            opencl_sys::CL_KERNEL_ARG_ACCESS_READ_WRITE => MemAccess::READ_WRITE,
            opencl_sys::CL_KERNEL_ARG_ACCESS_NONE => MemAccess::NONE,
            _ => unreachable!()
        };

        return Ok(v)
    }

    /// Returns the type name specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_type_name (&self, idx: u32) -> Result<String> {
        self.get_arg_info_string(CL_KERNEL_ARG_TYPE_NAME, idx)
    }

    /// Returns the type qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_qualifier (&self, idx: u32) -> Result<String> {
        self.get_arg_info(CL_KERNEL_ARG_TYPE_QUALIFIER, idx)
    }

    /// Returns the name specified for the argument given by ```idx```. 
    #[inline(always)]
    pub fn arg_name (&self, idx: u32) -> Result<String> {
        self.get_arg_info_string(CL_KERNEL_ARG_NAME, idx)
    }

    #[inline]
    fn get_arg_info_string (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetKernelArgInfo(self.id(), idx, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetKernelArgInfo(self.id(), idx, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_arg_info<T> (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetKernelArgInfo(self.id(), idx, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AddrQualifier {
    Global = CL_KERNEL_ARG_ADDRESS_GLOBAL,
    Local = CL_KERNEL_ARG_ADDRESS_LOCAL,
    Constant = CL_KERNEL_ARG_ADDRESS_CONSTANT,
    Private = CL_KERNEL_ARG_ADDRESS_PRIVATE
}

impl Default for AddrQualifier {
    #[inline(always)]
    fn default() -> Self {
        Self::Private
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct TypeQualifier: cl_kernel_arg_type_qualifier {
        const CONST = CL_KERNEL_ARG_TYPE_CONST;
        const RESTRICT = CL_KERNEL_ARG_TYPE_RESTRICT;
        const VOLATILE = CL_KERNEL_ARG_TYPE_VOLATILE;
    }
}

impl Default for TypeQualifier {
    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}