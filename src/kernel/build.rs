use opencl_sys::cl_mem;
use super::Kernel;
use crate::core::*;

pub struct Build<'a> {
    parent: &'a Kernel,
    args: Box<[Option<ArgumentType>]>
}

impl<'a> Build<'a> {
    #[inline(always)]
    pub fn new (parent: &'a Kernel) -> Result<Self> {
        let arg_count = parent.num_args()? as usize;
        todo!()
    }

    #[inline(always)]
    pub fn set_value<T: Copy> (&mut self, idx: usize, v: T) -> &mut Self {
        let mut bytes = Box::new_uninit_slice(core::mem::size_of::<T>());
        let ty;

        unsafe {
            core::ptr::copy_nonoverlapping(&v, bytes.as_mut_ptr().cast(), 1);
            ty = ArgumentType::Value(bytes.assume_init());
        }

        self.args[idx] = Some(ty);
        self
    }

    #[inline(always)]
    pub fn set_mem_buffer (&mut self, idx: usize, mem: cl_mem) -> &mut Self {
        self.args[idx] = Some(ArgumentType::Buffer(mem));
        self
    }

    #[inline(always)]
    pub fn set_alloc<T: Copy> (&mut self, idx: usize, count: usize) -> &mut Self {
        let bytes = count.checked_mul(core::mem::size_of::<T>()).unwrap();
        self.args[idx] = Some(ArgumentType::Alloc(bytes));
        self
    }
}

enum ArgumentType {
    Value (Box<[u8]>),
    Buffer (cl_mem),
    Alloc (usize)
}