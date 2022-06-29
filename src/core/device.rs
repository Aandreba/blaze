use core::{mem::MaybeUninit, intrinsics::transmute, num::{NonZeroUsize, NonZeroU32, NonZeroU64, IntErrorKind}, fmt::{Debug, Display}, str::FromStr};
use opencl_sys::{cl_device_id, clGetDeviceIDs, CL_DEVICE_TYPE_ALL, cl_device_info, clGetDeviceInfo, CL_DEVICE_PLATFORM, CL_DEVICE_ADDRESS_BITS, cl_bool, CL_DEVICE_AVAILABLE, CL_FP_DENORM, CL_FP_INF_NAN, CL_FP_ROUND_TO_NEAREST, CL_FP_ROUND_TO_ZERO, CL_FP_ROUND_TO_INF, cl_device_fp_config, CL_DEVICE_DOUBLE_FP_CONFIG, CL_DEVICE_ENDIAN_LITTLE, CL_DEVICE_ERROR_CORRECTION_SUPPORT, cl_device_exec_capabilities, CL_EXEC_KERNEL, CL_EXEC_NATIVE_KERNEL, CL_DEVICE_EXECUTION_CAPABILITIES, CL_DEVICE_EXTENSIONS, CL_DEVICE_GLOBAL_MEM_CACHE_SIZE, CL_NONE, CL_READ_ONLY_CACHE, cl_device_mem_cache_type, CL_DEVICE_GLOBAL_MEM_CACHE_TYPE, CL_READ_WRITE_CACHE, CL_DEVICE_GLOBAL_MEM_CACHELINE_SIZE, CL_DEVICE_GLOBAL_MEM_SIZE, CL_DEVICE_HALF_FP_CONFIG, CL_DEVICE_IMAGE_SUPPORT, CL_DEVICE_IMAGE2D_MAX_HEIGHT, CL_DEVICE_IMAGE2D_MAX_WIDTH, CL_DEVICE_IMAGE3D_MAX_WIDTH, CL_DEVICE_IMAGE3D_MAX_HEIGHT, CL_DEVICE_IMAGE3D_MAX_DEPTH, CL_DEVICE_LOCAL_MEM_SIZE, CL_LOCAL, CL_GLOBAL, CL_DEVICE_LOCAL_MEM_TYPE, CL_DEVICE_MAX_CLOCK_FREQUENCY, CL_DEVICE_MAX_COMPUTE_UNITS, CL_DEVICE_MAX_CONSTANT_ARGS, CL_DEVICE_MAX_CONSTANT_BUFFER_SIZE, CL_DEVICE_MAX_MEM_ALLOC_SIZE, CL_DEVICE_MAX_PARAMETER_SIZE, CL_DEVICE_MAX_READ_IMAGE_ARGS, CL_DEVICE_MAX_SAMPLERS, CL_DEVICE_MAX_WORK_GROUP_SIZE, CL_DEVICE_MAX_WORK_ITEM_DIMENSIONS, CL_DEVICE_MAX_WORK_ITEM_SIZES, CL_DEVICE_MAX_WRITE_IMAGE_ARGS, CL_DEVICE_MEM_BASE_ADDR_ALIGN, CL_DEVICE_MIN_DATA_TYPE_ALIGN_SIZE, CL_DEVICE_NAME, CL_DEVICE_PREFERRED_VECTOR_WIDTH_CHAR, CL_DEVICE_PREFERRED_VECTOR_WIDTH_SHORT, CL_DEVICE_PREFERRED_VECTOR_WIDTH_INT, CL_DEVICE_PREFERRED_VECTOR_WIDTH_LONG, CL_DEVICE_PREFERRED_VECTOR_WIDTH_FLOAT, CL_DEVICE_PREFERRED_VECTOR_WIDTH_DOUBLE, CL_DEVICE_PROFILE, CL_DEVICE_PROFILING_TIMER_RESOLUTION, CL_DEVICE_SINGLE_FP_CONFIG, cl_device_type, CL_DEVICE_TYPE_CPU, CL_DEVICE_TYPE_GPU, CL_DEVICE_TYPE_ACCELERATOR, CL_DEVICE_TYPE_CUSTOM, CL_DEVICE_TYPE, CL_DEVICE_VENDOR, CL_DEVICE_VENDOR_ID, CL_DEVICE_VERSION, CL_DRIVER_VERSION, cl_device_svm_capabilities, CL_DEVICE_SVM_COARSE_GRAIN_BUFFER, CL_DEVICE_SVM_FINE_GRAIN_BUFFER, CL_DEVICE_SVM_FINE_GRAIN_SYSTEM, CL_DEVICE_SVM_ATOMICS, CL_DEVICE_SVM_CAPABILITIES, cl_version, CL_VERSION_PATCH_BITS, CL_VERSION_MINOR_BITS, CL_VERSION_MAJOR_MASK, CL_VERSION_MINOR_MASK, CL_VERSION_PATCH_MASK};
use super::*;

lazy_static! {
    static ref DEVICES : Vec<Device> = unsafe {
        let mut result = Vec::<Device>::new();

        for platform in Platform::all() {
            let mut cnt = 0;
            tri_panic!(clGetDeviceIDs(platform.0, CL_DEVICE_TYPE_ALL, 0, core::ptr::null_mut(), &mut cnt));
            let cnt_size = usize::try_from(cnt).unwrap();

            result.reserve(cnt_size);
            tri_panic!(clGetDeviceIDs(platform.0, CL_DEVICE_TYPE_ALL, cnt, result.as_mut_ptr().add(result.len()).cast(), core::ptr::null_mut()));
            result.set_len(result.len() + cnt_size);
        }

        #[cfg(debug_assertions)]
        if !result.iter().all(|x| x.version().map(|x| x.major() >= 2).unwrap_or(true)) {
            eprintln!("WARNING: Some of the devices inside this context arn't OpenCL 2.0+ compatible. If this is intentional, we suggest you turn off the `cl2` feature");
        }

        result
    };
}

/// OpenCL device
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Device (pub(crate) cl_device_id);

impl Device {
    /// The default compute device address space size specified as an unsigned integer value in bits. Currently supported values are 32 or 64 bits.
    #[inline(always)]
    pub fn address_bits (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_ADDRESS_BITS)
    }

    /// Is ```true``` if the device is available and ```false``` if the device is not available.
    #[inline(always)]
    pub fn available (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_AVAILABLE)?;
        Ok(v != 0)
    }

    /// Describes the OPTIONAL double precision floating-point capability of the OpenCL device
    #[inline(always)]
    pub fn double_fp_config (&self) -> Result<FpConfig> {
        self.get_info_bits(CL_DEVICE_DOUBLE_FP_CONFIG)
    }

    /// Is ```true``` if the OpenCL device is a little endian device and ```false``` otherwise.
    #[inline(always)]
    pub fn endian_little (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_ENDIAN_LITTLE)?;
        Ok(v != 0)
    }

    /// Is ```true``` if the device implements error correction for the memories, caches, registers etc. in the device. Is ```false``` if the device does not implement error correction. This can be a requirement for certain clients of OpenCL.
    #[inline(always)]
    pub fn error_connection_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_ERROR_CORRECTION_SUPPORT)?;
        Ok(v != 0)
    }

    /// Describes the execution capabilities of the device
    #[inline(always)]
    pub fn execution_capabilities (&self) -> Result<ExecCapabilities> {
        self.get_info_bits(CL_DEVICE_EXECUTION_CAPABILITIES)
    }

    /// Returns a list of extension names
    #[inline(always)]
    pub fn extensions (&self) -> Result<Vec<String>> {
        Ok (
            self.get_info_string(CL_DEVICE_EXTENSIONS)?
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<_>>()
        )
    }

    /// Returns a space-separated list of extension names (the extension names themselves do not contain any spaces)
    #[inline(always)]
    pub fn extensions_string (&self) -> Result<String> {
        self.get_info_string(CL_DEVICE_EXTENSIONS)
    }

    /// Size of global memory cache in bytes.
    #[inline(always)]
    pub fn global_mem_cache_size (&self) -> Result<u64> {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_CACHE_SIZE)
    }

    /// Type of global memory cache supported.
    #[inline(always)]
    pub fn global_mem_cache_type (&self) -> Result<Option<MemCacheType>> {
        match self.get_info_bits::<cl_device_mem_cache_type>(CL_DEVICE_GLOBAL_MEM_CACHE_TYPE)? {
            CL_NONE => Ok(None),
            other => unsafe { Ok(Some(transmute(other))) }
        }
    }

    /// Size of global memory cache line in bytes.
    #[inline(always)]
    pub fn global_mem_cahceline_size (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_CACHELINE_SIZE)
    }

    /// Size of global memory in bytes.
    #[inline(always)]
    pub fn global_mem_size (&self) -> Result<u64> {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_SIZE)
    }

    /// Describes the OPTIONAL half precision floating-point capability of the OpenCL device
    #[inline(always)]
    pub fn half_fp_config (&self) -> Result<FpConfig> {
        self.get_info_bits(CL_DEVICE_HALF_FP_CONFIG)
    }
    
    /// Is ```true``` if images are supported by the OpenCL device and ```false``` otherwise.
    #[inline(always)]
    pub fn image_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_IMAGE_SUPPORT)?;
        Ok(v != 0)
    }

    /// Max height of 2D image in pixels. The minimum value is 8192 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image2d_max_height (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE2D_MAX_HEIGHT).map(NonZeroUsize::new)
    }

    /// Max width of 2D image in pixels. The minimum value is 8192 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image2d_max_width (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE2D_MAX_WIDTH).map(NonZeroUsize::new)
    }

    /// Max depth of 3D image in pixels. The minimum value is 2048 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image3d_max_depth (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE3D_MAX_DEPTH).map(NonZeroUsize::new)
    }

    /// Max height of 3D image in pixels. The minimum value is 2048 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image3d_max_height (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE3D_MAX_HEIGHT).map(NonZeroUsize::new)
    }

    /// Max width of 3D image in pixels. The minimum value is 2048 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image3d_max_width (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE3D_MAX_WIDTH).map(NonZeroUsize::new)
    }

    /// Size of local memory arena in bytes. The minimum value is 16 KB.
    #[inline(always)]
    pub fn local_mem_size (&self) -> Result<NonZeroU64> {
        unsafe {
            Ok(NonZeroU64::new_unchecked(self.get_info_bits::<u64>(CL_DEVICE_LOCAL_MEM_SIZE)?))
        }
    }

    /// Type of local memory supported.
    #[inline(always)]
    pub fn local_mem_type (&self) -> Result<LocalMemType> {
        self.get_info_bits(CL_DEVICE_LOCAL_MEM_TYPE)
    }

    /// Maximum configured clock frequency of the device in MHz.
    #[inline(always)]
    pub fn max_clock_frequency (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_MAX_CLOCK_FREQUENCY)
    }

    /// The number of parallel compute cores on the OpenCL device. The minimum value is 1.
    #[inline(always)]
    pub fn max_compute_units (&self) -> Result<NonZeroU32> {
        unsafe { 
            Ok(NonZeroU32::new_unchecked(self.get_info_bits::<u32>(CL_DEVICE_MAX_COMPUTE_UNITS)?))
        }
    }

    /// Max number of arguments declared with the ```__constant``` qualifier in a kernel. The minimum value is 8.
    #[inline(always)]
    pub fn max_constant_args (&self) -> Result<NonZeroU32> {
        unsafe { 
            Ok(NonZeroU32::new_unchecked(self.get_info_bits::<u32>(CL_DEVICE_MAX_CONSTANT_ARGS)?))
        }
    }

    /// Max size in bytes of a constant buffer allocation. The minimum value is 64 KB.
    #[inline(always)]
    pub fn max_constant_buffer_size (&self) -> Result<NonZeroU64> {
        unsafe { 
            Ok(NonZeroU64::new_unchecked(self.get_info_bits::<u64>(CL_DEVICE_MAX_CONSTANT_BUFFER_SIZE)?))
        }
    }

    /// Max size of memory object allocation in bytes. The minimum value is max (1/4th of [```global_mem_size```](), 128*1024*1024)
    #[inline(always)]
    pub fn max_mem_alloc_size (&self) -> Result<NonZeroU64> {
        unsafe { 
            Ok(NonZeroU64::new_unchecked(self.get_info_bits::<u64>(CL_DEVICE_MAX_MEM_ALLOC_SIZE)?))
        }
    }

    /// Max size in bytes of the arguments that can be passed to a kernel. The minimum value is 256.
    #[inline(always)]
    pub fn max_parameter_size (&self) -> Result<NonZeroUsize> {
        unsafe { 
            Ok(NonZeroUsize::new_unchecked(self.get_info_bits::<usize>(CL_DEVICE_MAX_PARAMETER_SIZE)?))
        }
    }

    /// Max number of simultaneous image objects that can be read by a kernel. The minimum value is 128 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn max_read_image_args (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits::<u32>(CL_DEVICE_MAX_READ_IMAGE_ARGS).map(NonZeroU32::new)
    }

    /// Maximum number of samplers that can be used in a kernel. The minimum value is 16 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn max_samplers (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits::<u32>(CL_DEVICE_MAX_SAMPLERS).map(NonZeroU32::new)
    }

    /// Maximum number of work-items in a work-group executing a kernel using the data parallel execution model. The minimum value is 1.
    #[inline(always)]
    pub fn max_work_group_size (&self) -> Result<NonZeroUsize> {
        unsafe {
            Ok(NonZeroUsize::new_unchecked(self.get_info_bits::<usize>(CL_DEVICE_MAX_WORK_GROUP_SIZE)?))
        }
    }

    /// Maximum dimensions that specify the global and local work-item IDs used by the data parallel execution model. The minimum value is 3.
    #[inline(always)]
    pub fn max_work_item_dimensions (&self) -> Result<NonZeroU32> {
        unsafe {
            Ok(NonZeroU32::new_unchecked(self.get_info_bits::<u32>(CL_DEVICE_MAX_WORK_ITEM_DIMENSIONS)?))
        }
    }

    /// Maximum number of work-items that can be specified in each dimension of the work-group to clEnqueueNDRangeKernel. Returns n ```usize``` entries, where n is the value returned by the query for [```max_work_item_dimensions```]. The minimum value is (1, 1, 1).
    #[inline(always)]
    pub fn max_work_item_sizes (&self) -> Result<Vec<NonZeroUsize>> {
        let n = usize::try_from(self.max_work_item_dimensions()?.get()).unwrap();
        // FIXME: maybe using nonzero ints messes up the alignment?
        let mut max_work_item_sizes = Vec::<NonZeroUsize>::with_capacity(n);

        let len = n.checked_mul(core::mem::size_of::<usize>()).expect("Integer multiplication oveflow. Too many work items to fit in a vector");
        unsafe {
            clGetDeviceInfo(self.0, CL_DEVICE_MAX_WORK_ITEM_SIZES, len, max_work_item_sizes.as_mut_ptr().cast(), core::ptr::null_mut());
            max_work_item_sizes.set_len(n);
        }

        Ok(max_work_item_sizes)
    }

    /// Max number of simultaneous image objects that can be written to by a kernel. The minimum value is 8 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn max_write_image_args (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits::<u32>(CL_DEVICE_MAX_WRITE_IMAGE_ARGS).map(NonZeroU32::new)
    }

    /// Describes the alignment in bits of the base address of any allocated memory object.
    #[inline(always)]
    pub fn mem_base_addr_align (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_MEM_BASE_ADDR_ALIGN)
    }

    /// The smallest alignment in bytes which can be used for any data type.
    #[inline(always)]
    pub fn min_data_type_align_size (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_MIN_DATA_TYPE_ALIGN_SIZE)
    }

    /// Device name string.
    #[inline(always)]
    pub fn name (&self) -> Result<String> {
        self.get_info_string(CL_DEVICE_NAME)
    }

    /// The platform associated with this device.
    #[inline(always)]
    pub fn platform (&self) -> Result<Platform> {
        self.get_info_bits(CL_DEVICE_PLATFORM)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_char (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_CHAR)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_short (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_SHORT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_int (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_INT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_long (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_LONG)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_float (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_FLOAT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector. if the ```cl_khr_fp64``` extension is not supported, it must return 0.
    #[inline(always)]
    pub fn preferred_vector_width_double (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_DOUBLE)
    }

    /// OpenCL profile string. Returns the profile name supported by the device (see note)
    #[inline(always)]
    pub fn profile (&self) -> String {
        self.get_info_string(CL_DEVICE_PROFILE).unwrap()
    }

    /// Describes the resolution of device timer. This is measured in nanoseconds.
    #[inline(always)]
    pub fn profiling_timer_resolution (&self) -> Result<usize> {
        self.get_info_bits(CL_DEVICE_PROFILING_TIMER_RESOLUTION)
    }

    /*
    /// Describes the command-queue properties supported by the device.
    #[inline(always)]
    pub fn queue_properties (&self) -> Result<CommandQueueProps> {
        self.get_info_bits(CL_DEVICE_QUEUE_PROPERTIES)
    }
    */

    #[cfg(feature = "cl2")]
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_REFERENCE_COUNT)
    }

    /// Describes single precision floating-point capability of the device.
    #[inline(always)]
    pub fn single_fp_config (&self) -> Result<FpConfig> {
        self.get_info_bits(CL_DEVICE_SINGLE_FP_CONFIG)
    }

    #[inline(always)]
    pub fn svm_capabilities (&self) -> Result<Option<SvmCapability>> {
        match self.get_info_bits(CL_DEVICE_SVM_CAPABILITIES) {
            Ok(x) => Ok(Some(x)),
            Err(Error::InvalidValue) => Ok(None),
            Err(e) => Err(e)
        }
    }

    /// The OpenCL device type.
    #[inline(always)]
    pub fn ty (&self) -> Result<DeviceType> {
        self.get_info_bits(CL_DEVICE_TYPE)
    }

    /// Vendor name string.
    #[inline(always)]
    pub fn vendor (&self) -> Result<String> {
        self.get_info_string(CL_DEVICE_VENDOR)
    }

    /// A unique device vendor identifier. An example of a unique device identifier could be the PCIe ID.
    #[inline(always)]
    pub fn vendor_id (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_VENDOR_ID)
    }

    /// OpenCL version string.
    #[inline(always)]
    pub fn version_string (&self) -> Result<String> {
        self.get_info_string(CL_DEVICE_VERSION)
    }

    /// OpenCL version
    pub fn version (&self) -> Result<Version> {
        let version = self.version_string()?;
        let section = version.split(' ').nth(1).ok_or(Error::InvalidValue)?;
        Version::from_str(section).map_err(|_| Error::InvalidValue)
    }

    /// OpenCL software driver version string in the form _major_number_._minor_number_.
    #[inline(always)]
    pub fn driver_version_string (&self) -> Result<String> {
        self.get_info_string(CL_DRIVER_VERSION)
    }

    /// OpenCL software driver version
    pub fn driver_version (&self) -> Result<Version> {
        let driver = self.driver_version_string()?;
        Version::from_str(&driver).map_err(|_| Error::InvalidValue)
    }

    #[inline(always)]
    pub fn has_f16 (&self) -> Result<bool> {
        let ext = self.extensions_string()?;
        Ok(ext.split_whitespace().any(|x| x == "cl_khr_fp16"))
    }

    #[inline(always)]
    pub fn has_f64 (&self) -> Result<bool> {
        let ext = self.extensions_string()?;
        Ok(ext.split_whitespace().any(|x| x == "cl_khr_fp64"))
    }
    
    #[inline(always)]
    pub fn all () -> &'static [Device] {
        &once_cell::sync::Lazy::force(&DEVICES)
    }

    #[inline(always)]
    pub fn first () -> Option<&'static Device> {
        DEVICES.first()
    }

    #[inline(always)]
    pub fn from_platform (platform: Platform) -> impl Iterator<Item = Device> {
        DEVICES.iter().cloned().filter_map(move |x| {
            match x.platform() {
                Ok(plat) if plat == platform => Some(x),
                _ => None
            }
        })
    }

    #[inline]
    fn get_info_string (&self, ty: cl_device_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetDeviceInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetDeviceInfo(self.0, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));

            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info_bits<T> (&self, ty: cl_device_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();

        unsafe {
            tri!(clGetDeviceInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Device")
        .field("id", &self.0)
        .field("name", &self.name())
        .field("vendor", &self.vendor())
        .field("type", &self.ty())
        .field("version", &self.version())
        .finish()
    }
}

#[cfg(feature = "cl1_2")]
impl Clone for Device {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(opencl_sys::clRetainDevice(self.0))
        }

        Self(self.0)
    }
}

#[cfg(feature = "cl1_2")]
impl Drop for Device {
    #[inline(always)]
    fn drop (&mut self) {
        unsafe {
            tri_panic!(opencl_sys::clReleaseDevice(self.0));
        }
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

bitflags::bitflags! {
    /// The OpenCL device type.
    #[repr(transparent)]
    pub struct DeviceType : cl_device_type {
        const CPU = CL_DEVICE_TYPE_CPU;
        const GPU = CL_DEVICE_TYPE_GPU;
        const ACCELERATOR = CL_DEVICE_TYPE_ACCELERATOR;
        const DEFAULT = CL_DEVICE_TYPE_CUSTOM;
    }

    /// Describes the floating-point capability of the OpenCL device.
    #[repr(transparent)]
    pub struct FpConfig : cl_device_fp_config {
        const DENORM = CL_FP_DENORM;
        const INF_NAN = CL_FP_INF_NAN;
        const ROUND_TO_NEAREST = CL_FP_ROUND_TO_NEAREST;
        const ROUND_TO_ZERO = CL_FP_ROUND_TO_ZERO;
        const ROUND_TO_INF = CL_FP_ROUND_TO_INF;
    }

    /// Describes the execution capabilities of the device
    #[repr(transparent)]
    pub struct ExecCapabilities : cl_device_exec_capabilities {
        const KERNEL = CL_EXEC_KERNEL;
        const NATIVE_KERNEL = CL_EXEC_NATIVE_KERNEL;
    }
}

/// Type of global memory cache supported.
#[derive(Debug)]
#[repr(u32)]
pub enum MemCacheType {
    ReadOnly = CL_READ_ONLY_CACHE,
    ReadWrite = CL_READ_WRITE_CACHE,
}

/// Type of local memory supported. This can be set to [```Self::Local```] implying dedicated local memory storage such as SRAM, or [```Self::Global```].
#[derive(Debug)]
#[repr(u32)]
pub enum LocalMemType {
    Local = CL_LOCAL,
    Global = CL_GLOBAL
}

bitflags::bitflags! {
    pub struct SvmCapability: cl_device_svm_capabilities {
        ///  Support for coarse-grain buffer sharing using clSVMAlloc. Memory consistency is guaranteed at synchronization points and the host must use calls to clEnqueueMapBuffer and clEnqueueUnmapMemObject.
        const COARSE_GRAIN_BUFFER = CL_DEVICE_SVM_COARSE_GRAIN_BUFFER;
        /// Support for fine-grain buffer sharing using clSVMAlloc. Memory consistency is guaranteed atsynchronization points without need for clEnqueueMapBuffer and clEnqueueUnmapMemObject.
        const FINE_GRAIN_BUFFER = CL_DEVICE_SVM_FINE_GRAIN_BUFFER;
        /// Support for sharing the host’s entire virtual memory including memory allocated using malloc. Memory consistency is guaranteed at synchronization points.
        const FINE_GRAIN_SYSTEM = CL_DEVICE_SVM_FINE_GRAIN_SYSTEM;
        /// Support for the OpenCL 2.0 atomic operations that provide memory consistency across the host and all OpenCL devices supporting fine-grain SVM allocations.
        const ATOMICS = CL_DEVICE_SVM_ATOMICS;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Version (cl_version);

impl Version {
    pub const CL1 : Self = Self::from_inner_parts(1, 0, 0);
    pub const CL1_1 : Self = Self::from_inner_parts(1, 1, 0);
    pub const CL1_2 : Self = Self::from_inner_parts(1, 2, 0);
    pub const CL2 : Self = Self::from_inner_parts(2, 0, 0);
    pub const CL2_1 : Self = Self::from_inner_parts(2, 1, 0);
    pub const CL2_2 : Self = Self::from_inner_parts(2, 2, 0);
    pub const CL3 : Self = Self::from_inner_parts(3, 0, 0);

    const MAJOR : u32 = CL_VERSION_MINOR_BITS + CL_VERSION_PATCH_BITS;

    #[inline(always)]
    pub const fn from_bits (bits : u32) -> Self {
        Self(bits)
    }

    #[inline(always)]
    pub const fn from_inner_parts (major: u32, minor: u32, patch: u32) -> Self {
        Self (
            ((major & CL_VERSION_MAJOR_MASK) << Self::MAJOR) |
            ((minor & CL_VERSION_MINOR_MASK) << CL_VERSION_PATCH_BITS) |
            (patch & CL_VERSION_PATCH_MASK)
        )
    }

    #[inline(always)]
    pub const fn into_inner_parts (self) -> (u32, u32, u32) {
        (self.major(), self.minor(), self.patch())
    }

    #[inline(always)]
    pub const fn major(&self) -> u32 {
        self.0 >> Self::MAJOR
    }

    #[inline(always)]
    pub const fn minor (&self) -> u32 {
        (self.0 >> CL_VERSION_PATCH_BITS) & CL_VERSION_MINOR_MASK
    }

    #[inline(always)]
    pub const fn patch (&self) -> u32 {
        self.0 & CL_VERSION_PATCH_MASK
    }
}

impl FromStr for Version {
    type Err = IntErrorKind;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut parts = s.split('.');
        
        let major = parts.next().ok_or(IntErrorKind::Empty)?.parse::<u32>().map_err(|e| e.kind().clone())?;
        let minor = parts.next().ok_or(IntErrorKind::Empty)?.parse::<u32>().map_err(|e| e.kind().clone())?;
        let patch_str = parts.next();

        let patch;
        if let Some(inner) = patch_str {
            patch = Some(inner.parse::<u32>().map_err(|e| e.kind().clone())?)
        } else {
            patch = None;
        }

        if parts.next().is_some() {
            return Err(IntErrorKind::InvalidDigit);
        }

        Ok(Self::from_inner_parts(major, minor, patch.unwrap_or_default()))
    }
}

impl Debug for Version {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for Version {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}