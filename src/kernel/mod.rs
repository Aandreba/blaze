flat_mod!(build);

use std::mem::MaybeUninit;
use opencl_sys::{cl_kernel, clCreateKernel, cl_kernel_info, CL_KERNEL_ARG_NAME, CL_KERNEL_ARG_TYPE_QUALIFIER, CL_KERNEL_ARG_TYPE_NAME, CL_KERNEL_ARG_ACCESS_QUALIFIER, CL_KERNEL_ARG_ADDRESS_QUALIFIER, clRetainProgram, CL_KERNEL_PROGRAM, cl_context, CL_KERNEL_CONTEXT, CL_KERNEL_REFERENCE_COUNT, CL_KERNEL_NUM_ARGS, CL_KERNEL_FUNCTION_NAME, CL_KERNEL_ARG_ADDRESS_GLOBAL, CL_KERNEL_ARG_ADDRESS_LOCAL, CL_KERNEL_ARG_ADDRESS_CONSTANT, CL_KERNEL_ARG_ADDRESS_PRIVATE, CL_KERNEL_ARG_ACCESS_READ_ONLY, CL_KERNEL_ARG_ACCESS_WRITE_ONLY, CL_KERNEL_ARG_ACCESS_READ_WRITE, CL_KERNEL_ARG_ACCESS_NONE, cl_kernel_arg_type_qualifier, CL_KERNEL_ARG_TYPE_CONST, CL_KERNEL_ARG_TYPE_RESTRICT, CL_KERNEL_ARG_TYPE_VOLATILE, clGetKernelInfo, cl_kernel_arg_info, clGetKernelArgInfo};
use crate::core::*;

#[repr(transparent)]
pub struct Kernel (pub(crate) cl_kernel);

impl Kernel {
    #[inline]
    pub fn new (prog: &Program, name: &str) -> Result<Self> {
        let mut name = name.as_bytes().to_vec();
        name.push(0);
        
        let mut err = 0;
        let id = unsafe { clCreateKernel(prog.0, name.as_ptr().cast(), &mut err) };

        if err != 0 { return Err(Error::from(err)) }
        Ok(Self(id))
    }

    #[inline(always)]
    pub fn build (&self) -> Result<Build<'_>> {
        Build::new(self)
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
    pub unsafe fn context_id (&self) -> Result<cl_context> {
        let ctx : cl_context = self.get_info(CL_KERNEL_CONTEXT)?;
        Ok(ctx)
    }

    /// Return the program object associated with _kernel_.
    #[inline(always)]
    pub fn program (&self) -> Result<Program> {
        let prog : Program = self.get_info(CL_KERNEL_PROGRAM)?;
        unsafe { tri_panic!(clRetainProgram(prog.0)); }
        Ok(prog)
    }

    /// Returns the address qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_address_qualifier (&self, idx: u32) -> Result<AddrQualifier> {
        self.get_arg_info(CL_KERNEL_ARG_ADDRESS_QUALIFIER, idx)
    }

    /// Returns the access qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_access_qualifier (&self, idx: u32) -> Result<AccessQualifier> {
        self.get_arg_info(CL_KERNEL_ARG_ACCESS_QUALIFIER, idx)
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
    fn get_info_string (&self, ty: cl_kernel_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetKernelInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetKernelInfo(self.0, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));

            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_kernel_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetKernelInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }

    #[inline]
    fn get_arg_info_string (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetKernelArgInfo(self.0, idx, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetKernelArgInfo(self.0, idx, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_arg_info<T> (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetKernelArgInfo(self.0, idx, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AccessQualifier {
    ReadOnly = CL_KERNEL_ARG_ACCESS_READ_ONLY,
    WriteOnly = CL_KERNEL_ARG_ACCESS_WRITE_ONLY,
    ReadWrite = CL_KERNEL_ARG_ACCESS_READ_WRITE,
    None = CL_KERNEL_ARG_ACCESS_NONE
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