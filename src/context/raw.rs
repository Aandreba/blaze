use std::{ptr::{addr_of_mut, NonNull}, ffi::c_void, mem::MaybeUninit};
use opencl_sys::{cl_context, clCreateContext, clCreateContextFromType, clRetainContext, clReleaseContext, cl_context_info, clGetContextInfo, CL_CONTEXT_REFERENCE_COUNT, CL_CONTEXT_DEVICES, cl_device_id, cl_context_properties, CL_CONTEXT_PROPERTIES};
use rscl_proc::docfg;
use crate::core::{*, device::DeviceType};
use super::ContextProperties;

#[repr(transparent)]
pub struct RawContext (NonNull<c_void>);

impl RawContext {
    pub fn new (props: ContextProperties, devices: &[Device]) -> Result<Self> {
        let num_devices = u32::try_from(devices.len()).unwrap();
        let props = props.to_bits();
        let props = match props {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let mut err = 0;
        let id = unsafe {
            clCreateContext(props, num_devices, devices.as_ptr().cast(), None, core::ptr::null_mut(), addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(Self::from_id(id).unwrap())
    }

    pub fn from_type (props: ContextProperties, ty: DeviceType) -> Result<Self> {
        let props = props.to_bits();
        let props = match props {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let mut err = 0;
        let id = unsafe {
            clCreateContextFromType(props, ty.bits(), None, core::ptr::null_mut(), addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(Self::from_id(id).unwrap())
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (v: cl_context) -> Self {
        Self(NonNull::new_unchecked(v))
    }

    #[inline(always)]
    pub const fn from_id (v: cl_context) -> Option<Self> {
        NonNull::new(v).map(Self)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_context {
        self.0.as_ptr()
    }

    /// Return the context reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_CONTEXT_REFERENCE_COUNT)
    }

    /// Return the number of devices in context.
    #[inline]
    pub fn num_devices (&self) -> Result<u32> {
        #[cfg(feature = "cl1_1")]
        if let Ok(x) = self.get_info(opencl_sys::CL_CONTEXT_NUM_DEVICES) {
            return Ok(x);
        }

        let mut res = 0;
        unsafe {
            tri!(clGetContextInfo(self.id(), CL_CONTEXT_DEVICES, 0, core::ptr::null_mut(), addr_of_mut!(res)))
        }

        let res = u32::try_from(res / core::mem::size_of::<cl_device_id>()).unwrap();
        Ok(res)
    }

    /// Return the list of devices and sub-devices in context.
    #[inline(always)]
    pub fn devices (&self) -> Result<Box<[Device]>> {
        self.get_info_array(CL_CONTEXT_DEVICES)
    }

    /// Return the properties argument specified in creation
    #[inline(always)]
    pub fn properties (&self) -> Result<ContextProperties> {
        let v = self.get_info_array::<cl_context_properties>(CL_CONTEXT_PROPERTIES)?;
        Ok(ContextProperties::from_bits(&v))
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_context_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetContextInfo(self.id(), ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }

    #[inline]
    fn get_info_array<T> (&self, ty: cl_context_info) -> Result<Box<[T]>> {
        let mut size = 0;
        unsafe {
            tri!(clGetContextInfo(self.id(), ty, 0, core::ptr::null_mut(), addr_of_mut!(size)))
        }

        let mut result = Box::<[T]>::new_uninit_slice(size / core::mem::size_of::<T>());
        unsafe {
            tri!(clGetContextInfo(self.id(), ty, size, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

#[docfg(feature = "cl3")]
impl RawContext {
    #[inline(always)]
    pub fn on_destruct (&self, f: impl 'static + FnOnce(RawContext)) -> Result<()> {
        let f = Box::new(f) as Box<_>;
        self.on_destruct_boxed(f)
    }

    #[inline(always)]
    pub fn on_destruct_boxed (&self, f: Box<dyn FnOnce(RawContext)>) -> Result<()> {
        let data = Box::into_raw(Box::new(f));
        unsafe { self.on_destruct_raw(destructor_callback, data.cast()) }
    }

    #[inline(always)]
    pub unsafe fn on_destruct_raw (&self, f: unsafe extern "C" fn(context: cl_context, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        tri!(opencl_sys::clSetContextDestructorCallback(self.id(), Some(f), user_data));
        todo!()
    }
}

impl Clone for RawContext {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainContext(self.id()))
        }

        Self(self.0.clone())
    }
}

impl Drop for RawContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseContext(self.id()))
        }   
    }
}

unsafe impl Send for RawContext {}
unsafe impl Sync for RawContext {}

#[cfg(feature = "cl3")]
unsafe extern "C" fn destructor_callback (context: cl_context, user_data: *mut c_void) {
    let f = *Box::from_raw(user_data as *mut Box<dyn FnOnce(RawContext)>);
    let context = RawContext::from_id_unchecked(context);
    f(context)
}