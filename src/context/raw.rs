use super::ContextProperties;
use crate::{
    core::{device::DeviceType, *},
    non_null_const,
    prelude::device::Version,
};
use blaze_proc::docfg;
use box_iter::BoxIntoIter;
use opencl_sys::*;
use std::ffi::CStr;
use std::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{addr_of_mut, NonNull},
};
use thinnbox::ThinBox;

/// A raw OpenCL context
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawContext(NonNull<c_void>);

impl RawContext {
    #[inline(always)]
    pub fn new(props: ContextProperties, devices: &[RawDevice]) -> Result<Self> {
        Self::inner_new::<fn(&CStr)>(
            props,
            devices,
            #[cfg(feature = "cl3")]
            None,
        )
    }

    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn with_logger<F: 'static + Fn(&CStr) + Send>(
        props: ContextProperties,
        devices: &[RawDevice],
        loger: F,
    ) -> Result<Self> {
        Self::inner_new(
            props,
            devices,
            #[cfg(feature = "cl3")]
            Some(loger),
        )
    }

    fn inner_new<F: 'static + Fn(&CStr) + Send>(
        props: ContextProperties,
        devices: &[RawDevice],
        #[cfg(feature = "cl3")] loger: Option<F>,
    ) -> Result<Self> {
        let num_devices = u32::try_from(devices.len()).unwrap();
        let props = props.to_bits();
        let props = match props {
            Some(x) => x.as_ptr(),
            None => core::ptr::null(),
        };

        cfg_if::cfg_if! {
            if #[cfg(feature = "cl3")] {
                let (pfn_notify, user_data) : (Option<unsafe extern "C" fn(*const std::ffi::c_char, *const c_void, usize, *mut c_void)>, Option<ThinBox<dyn Fn(&CStr) + Send>>) = match loger {
                    Some(x) => {
                        let f = ThinBox::<dyn 'static + Fn(&CStr) + Send>::new_unsize(x);
                        (Some(context_error), Some(f))
                    },

                    None => (None, None)
                };
            } else {
                let (pfn_notify, user_data) : (Option<unsafe extern "C" fn(*const std::ffi::c_char, *const c_void, usize, *mut c_void)>, Option<ThinBox<dyn Fn(&CStr) + Send>>) = (None, None);
            }
        }

        let user_data_ptr = match user_data {
            Some(ref x) => unsafe { x.as_raw().as_ptr() as *mut c_void },
            None => core::ptr::null_mut(),
        };

        let mut err = 0;
        let id = unsafe {
            clCreateContext(
                props,
                num_devices,
                devices.as_ptr().cast(),
                pfn_notify,
                user_data_ptr,
                addr_of_mut!(err),
            )
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        let this = unsafe { Self::from_id(id).unwrap() };

        #[cfg(feature = "cl3")]
        this.on_destruct(move || drop(user_data))?;

        Ok(this)
    }

    pub fn from_type(props: ContextProperties, ty: DeviceType) -> Result<Self> {
        let props = props.to_bits();
        let props = match props {
            Some(x) => x.as_ptr(),
            None => core::ptr::null(),
        };

        let mut err = 0;
        let id = unsafe {
            clCreateContextFromType(
                props,
                ty.bits(),
                None,
                core::ptr::null_mut(),
                addr_of_mut!(err),
            )
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        unsafe { Ok(Self::from_id(id).unwrap()) }
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked(v: cl_context) -> Self {
        Self(NonNull::new_unchecked(v))
    }

    #[inline(always)]
    pub const unsafe fn from_id(v: cl_context) -> Option<Self> {
        match non_null_const(v) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    #[inline(always)]
    pub const fn id(&self) -> cl_context {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub unsafe fn retain(&self) -> Result<()> {
        tri!(clRetainContext(self.id()));
        Ok(())
    }

    /// Return the context reference count.
    #[inline(always)]
    pub fn reference_count(&self) -> Result<u32> {
        self.get_info(CL_CONTEXT_REFERENCE_COUNT)
    }

    /// Return the number of devices in context.
    #[inline]
    pub fn num_devices(&self) -> Result<u32> {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "cl1_1", feature = "strict"))] {
                self.get_info(opencl_sys::CL_CONTEXT_NUM_DEVICES)
            } else {
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
        }
    }

    /// Return the list of devices and sub-devices in context.
    #[inline(always)]
    pub fn devices(&self) -> Result<Vec<RawDevice>> {
        let devices = self.get_info_array::<cl_device_id>(CL_CONTEXT_DEVICES)?;
        Ok(devices
            .into_iter()
            .map(|id| unsafe { RawDevice::from_id(id).unwrap() })
            .collect())
    }

    /// Returns the greatest common OpenCL version of this context's devices.
    #[inline]
    pub fn greatest_common_version(&self) -> Result<Version> {
        let devices = self.devices()?;
        let mut result = None;

        for device in devices.into_iter() {
            let version = device.version()?;

            if let Some(ref mut result) = result {
                if &version < result {
                    *result = version
                }

                continue;
            }

            result = Some(version)
        }

        Ok(result.unwrap_or_else(|| Version::CL1))
    }

    /// Return the properties argument specified in creation
    #[inline(always)]
    pub fn properties(&self) -> Result<ContextProperties> {
        let v = self.get_info_array::<cl_context_properties>(CL_CONTEXT_PROPERTIES)?;
        Ok(ContextProperties::from_bits(&v))
    }

    /// Get the list of image formats supported by an OpenCL implementation.
    #[cfg(feature = "image")]
    pub fn supported_image_formats(
        &self,
        access: crate::buffer::flags::MemAccess,
        ty: crate::memobj::MemObjectType,
    ) -> Result<Vec<crate::image::ImageFormat>> {
        use crate::image::ImageFormat;

        let mut size = 0;
        unsafe {
            tri!(clGetSupportedImageFormats(
                self.id(),
                access.to_bits(),
                ty as u32,
                0,
                core::ptr::null_mut(),
                addr_of_mut!(size)
            ))
        }

        let len = size as usize;
        let mut values = Box::<[cl_image_format]>::new_uninit_slice(len);
        let values = unsafe {
            tri!(clGetSupportedImageFormats(
                self.id(),
                access.to_bits(),
                ty as u32,
                size,
                values.as_mut_ptr().cast(),
                core::ptr::null_mut()
            ));
            values.assume_init()
        };

        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let v = match ImageFormat::from_raw(values[i]) {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::InvalidImageFormatDescriptor,
                        format!("{e:?}"),
                    ))
                }
            };

            result.push(v)
        }

        Ok(result)
    }

    #[inline]
    fn get_info<T: Copy>(&self, ty: cl_context_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();

        unsafe {
            tri!(clGetContextInfo(
                self.id(),
                ty,
                core::mem::size_of::<T>(),
                value.as_mut_ptr().cast(),
                core::ptr::null_mut()
            ));
            Ok(value.assume_init())
        }
    }

    #[inline]
    fn get_info_array<T: Copy>(&self, ty: cl_context_info) -> Result<Box<[T]>> {
        let mut size = 0;
        unsafe {
            tri!(clGetContextInfo(
                self.id(),
                ty,
                0,
                core::ptr::null_mut(),
                addr_of_mut!(size)
            ))
        }

        let mut result;
        cfg_if::cfg_if! {
            if #[cfg(feature = "nightly")] {
                result = Box::<[T]>::new_uninit_slice(size / core::mem::size_of::<T>());
            } else {
                let mut vec = Vec::<MaybeUninit<T>>::with_capacity(size / core::mem::size_of::<T>());
                unsafe { vec.set_len(vec.capacity()) };
                result = vec.into_boxed_slice();
            }
        }

        unsafe {
            tri!(clGetContextInfo(
                self.id(),
                ty,
                size,
                result.as_mut_ptr().cast(),
                core::ptr::null_mut()
            ));

            cfg_if::cfg_if! {
                if #[cfg(feature = "nightly")] {
                    Ok(result.assume_init())
                } else {
                    Ok(Box::from_raw(Box::into_raw(result) as *mut [T]))
                }
            }
        }
    }
}

#[docfg(feature = "cl3")]
impl RawContext {
    #[inline(always)]
    pub fn on_destruct(&self, f: impl 'static + FnOnce() + Send) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                let f = ThinBox::<dyn 'static + FnMut() + Send>::from_once(f);
            } else {
                let f = unsafe { ThinBox::<dyn 'static + FnMut() + Send>::from_once_unchecked(f) };
            }
        }

        unsafe {
            let user_data = ThinBox::into_raw(f);
            if let Err(e) = self.on_destruct_raw(destructor_callback, user_data.as_ptr().cast()) {
                let _ = ThinBox::<dyn 'static + FnMut() + Send>::from_raw(user_data);
                return Err(e);
            }
            return Ok(());
        }
    }

    #[inline(always)]
    pub unsafe fn on_destruct_raw(
        &self,
        f: unsafe extern "C" fn(context: cl_context, user_data: *mut c_void),
        user_data: *mut c_void,
    ) -> Result<()> {
        tri!(opencl_sys::clSetContextDestructorCallback(
            self.id(),
            Some(f),
            user_data
        ));
        Ok(())
    }
}

impl Clone for RawContext {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { tri_panic!(clRetainContext(self.id())) }

        Self(self.0.clone())
    }
}

impl Drop for RawContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { tri_panic!(clReleaseContext(self.id())) }
    }
}

unsafe impl Send for RawContext {}
unsafe impl Sync for RawContext {}

#[doc(hidden)]
#[cfg(feature = "cl3")]
unsafe extern "C" fn destructor_callback(_context: cl_context, user_data: *mut c_void) {
    use thinnbox::ThinBox;

    let mut f =
        ThinBox::<dyn 'static + FnMut() + Send>::from_raw(NonNull::new_unchecked(user_data.cast()));
    f()
}

#[doc(hidden)]
#[cfg(feature = "cl3")]
unsafe extern "C" fn context_error(
    errinfo: *const std::ffi::c_char,
    _private_info: *const c_void,
    _cb: usize,
    user_data: *mut c_void,
) {
    use thinnbox::ThinBox;

    let f = ThinBox::<dyn 'static + Fn(&CStr) + Send>::ref_from_raw(NonNull::new_unchecked(
        user_data.cast(),
    )) as &(dyn 'static + Fn(&CStr) + Send);
    f(std::ffi::CStr::from_ptr(errinfo))
}
