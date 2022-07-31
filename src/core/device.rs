use core::{mem::MaybeUninit, num::{NonZeroUsize, NonZeroU32, NonZeroU64, IntErrorKind}, fmt::{Debug, Display}, str::FromStr};
use std::{ptr::{NonNull}, ffi::c_void};
use opencl_sys::*;
use blaze_proc::docfg;
use crate::buffer::flags::MemAccess;
use super::*;

lazy_static! {
    static ref DEVICES : Vec<RawDevice> = unsafe {
        let mut result = Vec::<RawDevice>::new();

        for platform in RawPlatform::all() {
            let mut cnt = 0;
            tri_panic!(clGetDeviceIDs(platform.id(), CL_DEVICE_TYPE_ALL, 0, core::ptr::null_mut(), &mut cnt));
            let cnt_size = usize::try_from(cnt).unwrap();

            result.reserve(cnt_size);
            tri_panic!(clGetDeviceIDs(platform.id(), CL_DEVICE_TYPE_ALL, cnt, result.as_mut_ptr().add(result.len()).cast(), core::ptr::null_mut()));
            result.set_len(result.len() + cnt_size);
        }

        result
    };
}

/// OpenCL device
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawDevice (NonNull<c_void>);

impl RawDevice {
    #[inline(always)]
    pub const fn id (&self) -> cl_device_id {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const unsafe fn from_id (id: cl_device_id) -> Option<Self> {
        NonNull::new(id).map(Self)
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_device_id) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    /// The default compute device address space size specified as an unsigned integer value in bits. Currently supported values are 32 or 64 bits.
    #[inline(always)]
    pub fn address_bits (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_ADDRESS_BITS)
    }

    /// Describes the various memory orders and scopes that the device supports for atomic memory operations.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn atomic_memory_capabilities (&self) -> Result<Option<AtomicCapabilities>> {
        let v = self.get_info_bits::<opencl_sys::cl_device_atomic_capabilities>(opencl_sys::CL_DEVICE_ATOMIC_MEMORY_CAPABILITIES)?;
        Ok(AtomicCapabilities::from_bits(v))
    }

    /// Describes the various memory orders and scopes that the device supports for atomic fence operations.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn atomic_fence_capabilities (&self) -> Result<Option<AtomicCapabilities>> {
        let v = self.get_info_bits::<opencl_sys::cl_device_atomic_capabilities>(opencl_sys::CL_DEVICE_ATOMIC_FENCE_CAPABILITIES)?;
        Ok(AtomicCapabilities::from_bits(v))
    }    

    /// Is ```true``` if the device is available and ```false``` if the device is not available.
    #[inline(always)]
    pub fn available (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_AVAILABLE)?;
        Ok(v != 0)
    }

    /// A list of built-in kernels supported by the device. An empty list is returned if no built-in kernels are supported by the device.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn built_in_kernels (&self) -> Result<Vec<String>> {
        Ok(self.built_in_kernels_string()?
            .split(';')
            .map(str::trim)
            .map(str::to_string)
            .collect::<Vec<_>>())
    }

    /// A semi-colon separated list of built-in kernels supported by the device. An empty string is returned if no built-in kernels are supported by the device.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn built_in_kernels_string (&self) -> Result<String> {
        self.get_info_string(opencl_sys::CL_DEVICE_BUILT_IN_KERNELS)
    }

    /// Is ```false``` if the implementation does not have a compiler available to compile the program source. Is ```true``` if the compiler is available. This can be CL_FALSE for the embedded platform profile only.
    #[inline(always)]
    pub fn compiler_available (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_COMPILER_AVAILABLE)?;
        Ok(v != 0)
    }

    /// Describes device-side enqueue capabilities of the device.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn device_enqueue_capabilities (&self) -> Result<Option<DeviceEnqueueCapabilities>> {
        let v = self.get_info_bits::<opencl_sys::cl_device_device_enqueue_capabilities>(opencl_sys::CL_DEVICE_DEVICE_ENQUEUE_CAPABILITIES)?;
        Ok(DeviceEnqueueCapabilities::from_bits(v))
    }

    /// Describes the OPTIONAL double precision floating-point capability of the OpenCL device
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn double_fp_config (&self) -> Result<FpConfig> {
        self.get_info_bits(opencl_sys::CL_DEVICE_DOUBLE_FP_CONFIG)
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

    /// Is ```true``` if the device supports the generic address space and its associated built-in functions, and ```false``` otherwise.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn generic_address_space_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_GENERIC_ADDRESS_SPACE_SUPPORT)?;
        Ok(v != 0)
    }

    /// Size of global memory cache in bytes.
    #[inline(always)]
    pub fn global_mem_cache_size (&self) -> Result<u64> {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_CACHE_SIZE)
    }

    /// Type of global memory cache supported.
    #[inline(always)]
    pub fn global_mem_cache_type (&self) -> Result<MemAccess> {
        match self.get_info_bits::<cl_device_mem_cache_type>(CL_DEVICE_GLOBAL_MEM_CACHE_TYPE)? {
            CL_NONE => Ok(MemAccess::NONE),
            CL_READ_ONLY_CACHE => Ok(MemAccess::READ_ONLY),
            CL_READ_WRITE_CACHE => Ok(MemAccess::READ_WRITE),
            _ => unreachable!()
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

    /// Maximum preferred total size, in bytes, of all program variables in the global address space. This is a performance hint. An implementation may place such variables in storage with optimized device access. This query returns the capacity of such storage. The minimum value is 0.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn global_variable_preferred_total_size (&self) -> Result<usize> {
        self.get_info_bits(opencl_sys::CL_DEVICE_GLOBAL_VARIABLE_PREFERRED_TOTAL_SIZE)
    } 

    /// Describes the OPTIONAL half precision floating-point capability of the OpenCL device
    #[inline(always)]
    pub fn half_fp_config (&self) -> Result<FpConfig> {
        self.get_info_bits(CL_DEVICE_HALF_FP_CONFIG)
    }

    /// Is ```true``` if the device and the host have a unified memory subsystem and is ```false``` otherwise.
    #[docfg(feature = "cl1_1")]
    #[cfg_attr(feature = "cl2", deprecated)]
    #[inline(always)]
    pub fn host_unified_memory (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_HOST_UNIFIED_MEMORY)?;
        Ok(v != 0)
    }

    /// The intermediate languages that can be supported by clCreateProgramWithIL for this device.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn il_version (&self) -> Result<String> {
        self.get_info_string(opencl_sys::CL_DEVICE_IL_VERSION)
    }
    
    /// Is ```true``` if images are supported by the OpenCL device and ```false``` otherwise.
    #[inline(always)]
    pub fn image_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(CL_DEVICE_IMAGE_SUPPORT)?;
        Ok(v != 0)
    }

    /// Max number of images in a 1D or 2D image array. The minimum value is 2048 if CL_DEVICE_IMAGE_SUPPORT is CL_TRUE, the value is 0 otherwise.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn image_max_array_size (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_IMAGE_MAX_ARRAY_SIZE).map(NonZeroUsize::new)
    }

    /// Max number of pixels for a 1D image created from a buffer object. The minimum value is 65536 if CL_DEVICE_IMAGE_SUPPORT is CL_TRUE, the value is 0 otherwise.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn image_max_buffer_size (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_IMAGE_MAX_BUFFER_SIZE).map(NonZeroUsize::new)
    }

    /// The row pitch alignment size in pixels for 2D images created from a buffer. The value returned must be a power of 2.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn image_pitch_alignment (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_IMAGE_PITCH_ALIGNMENT).map(NonZeroU32::new)
    }

    /// This query specifies the minimum alignment in pixels of the host_ptr specified to clCreateBuffer or clCreateBufferWithProperties when a 2D image is created from a buffer which was created using CL_MEM_USE_HOST_PTR. The value returned must be a power of 2.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn image_base_address_alignment (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_IMAGE_PITCH_ALIGNMENT).map(NonZeroU32::new)
    }

    /// Max height of 2D image in pixels. The minimum value is 8192 if [`image_support`](RawDevice::image_support) is ```true```.
    #[inline(always)]
    pub fn image2d_max_height (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE2D_MAX_HEIGHT).map(NonZeroUsize::new)
    }

    /// Max width of 2D image in pixels. The minimum value is 8192 if [`image_support`](RawDevice::image_support) is ```true```.
    #[inline(always)]
    pub fn image2d_max_width (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE2D_MAX_WIDTH).map(NonZeroUsize::new)
    }

    /// Max depth of 3D image in pixels. The minimum value is 2048 if [`image_support`](RawDevice::image_support) is ```true```.
    #[inline(always)]
    pub fn image3d_max_depth (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE3D_MAX_DEPTH).map(NonZeroUsize::new)
    }

    /// Max height of 3D image in pixels. The minimum value is 2048 if [`image_support`](RawDevice::image_support) is ```true```.
    #[inline(always)]
    pub fn image3d_max_height (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE3D_MAX_HEIGHT).map(NonZeroUsize::new)
    }

    /// Max width of 3D image in pixels. The minimum value is 2048 if [`image_support`](RawDevice::image_support) is ```true```.
    #[inline(always)]
    pub fn image3d_max_width (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits::<usize>(CL_DEVICE_IMAGE3D_MAX_WIDTH).map(NonZeroUsize::new)
    }

    /// Returns the latest version of the conformance test suite that this device has fully passed in accordance with the official conformance process.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn latest_conformance_version_passed (&self) -> Result<String> {
        self.get_info_string(opencl_sys::CL_DEVICE_LATEST_CONFORMANCE_VERSION_PASSED)
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

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn linker_available (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_LINKER_AVAILABLE)?;
        Ok(v != 0)
    }

    /// Maximum configured clock frequency of the device in MHz.
    #[docfg(feature = "cl2_2")]
    #[inline(always)]
    pub fn max_clock_frequency (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_MAX_CLOCK_FREQUENCY)
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

    /// The maximum number of bytes of storage that may be allocated for any single variable in program scope or inside a function in an OpenCL kernel language declared in the global address space.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn max_global_variable_size (&self) -> Result<Option<NonZeroUsize>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_MAX_GLOBAL_VARIABLE_SIZE).map(NonZeroUsize::new)
    }

    /// Max size of memory object allocation in bytes. The minimum value is max (1/4th of [```global_mem_size```](), 128*1024*1024)
    #[inline(always)]
    pub fn max_mem_alloc_size (&self) -> Result<NonZeroU64> {
        unsafe { 
            Ok(NonZeroU64::new_unchecked(self.get_info_bits::<u64>(CL_DEVICE_MAX_MEM_ALLOC_SIZE)?))
        }
    }

    /// Maximum number of sub-groups in a work-group that a device is capable of executing on a single compute unit, for any given kernel-instance running on the device. The minimum value is 1 if the device supports subgroups, and must be 0 for devices that do not support subgroups. Support for subgroups is required for an OpenCL 2.1 or 2.2 device.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn max_num_sub_groups (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_MAX_NUM_SUB_GROUPS).map(NonZeroU32::new)
    }

    /// The maximum number of events in use by a device queue. These refer to events returned by the enqueue_ built-in functions to a device queue or user events returned by the create_user_event built-in function that have not been released. The minimum value is 1024 for devices supporting on-device queues, and must be 0 for devices that do not support on-device queues.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn max_on_device_events (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_MAX_ON_DEVICE_EVENTS).map(NonZeroU32::new)
    }

    /// The maximum number of device queues that can be created for this device in a single context. The minimum value is 1 for devices supporting on-device queues, and must be 0 for devices that do not support on-device queues.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn max_on_device_queues (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_MAX_ON_DEVICE_QUEUES).map(NonZeroU32::new)
    }

    /// Max size in bytes of the arguments that can be passed to a kernel. The minimum value is 256.
    #[inline(always)]
    pub fn max_parameter_size (&self) -> Result<NonZeroUsize> {
        unsafe { 
            Ok(NonZeroUsize::new_unchecked(self.get_info_bits::<usize>(CL_DEVICE_MAX_PARAMETER_SIZE)?))
        }
    }

    /// The maximum number of pipe objects that can be passed as arguments to a kernel. The minimum value is 16 for devices supporting pipes, and must be 0 for devices that do not support pipes.
    #[docfg(featurew = "cl2")]
    #[inline(always)]
    pub fn max_pipe_args (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_MAX_PIPE_ARGS).map(NonZeroU32::new)
    }

    /// Max number of simultaneous image objects that can be read by a kernel. The minimum value is 128 if [`image_support`](RawDevice::image_support) is ```true```.
    #[inline(always)]
    pub fn max_read_image_args (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits::<u32>(CL_DEVICE_MAX_READ_IMAGE_ARGS).map(NonZeroU32::new)
    }

    /// Max number of image objects arguments of a kernel declared with the write_only or read_write qualifier.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn max_read_write_image_args (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits::<u32>(opencl_sys::CL_DEVICE_MAX_READ_IMAGE_ARGS).map(NonZeroU32::new)
    }

    /// Maximum number of samplers that can be used in a kernel. The minimum value is 16 if [`image_support`](RawDevice::image_support) is ```true```.
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

    /// Maximum number of work-items that can be specified in each dimension of the work-group to clEnqueueNDRangeKernel. Returns n ```usize``` entries, where n is the value returned by the query for [`max_work_item_dimensions`](RawDevice::max_work_item_dimensions). The minimum value is (1, 1, 1).
    #[inline(always)]
    pub fn max_work_item_sizes (&self) -> Result<Vec<NonZeroUsize>> {
        let n = usize::try_from(self.max_work_item_dimensions()?.get()).unwrap();
        // FIXME: maybe using nonzero ints messes up the alignment?
        let mut max_work_item_sizes = Vec::<NonZeroUsize>::with_capacity(n);

        let len = n.checked_mul(core::mem::size_of::<usize>()).expect("Integer multiplication oveflow. Too many work items to fit in a vector");
        unsafe {
            clGetDeviceInfo(self.id(), CL_DEVICE_MAX_WORK_ITEM_SIZES, len, max_work_item_sizes.as_mut_ptr().cast(), core::ptr::null_mut());
            max_work_item_sizes.set_len(n);
        }

        Ok(max_work_item_sizes)
    }

    /// Max number of simultaneous image objects that can be written to by a kernel. The minimum value is 8 if [`image_support`](RawDevice::image_support) is ```true```.
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
    #[cfg_attr(feature = "cl1_2", deprecated)]
    #[inline(always)]
    pub fn min_data_type_align_size (&self) -> Result<u32> {
        self.get_info_bits(CL_DEVICE_MIN_DATA_TYPE_ALIGN_SIZE)
    }

    /// Device name string.
    #[inline(always)]
    pub fn name (&self) -> Result<String> {
        self.get_info_string(CL_DEVICE_NAME)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn native_vector_width_char (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_CHAR)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn native_vector_width_short (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_SHORT)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn native_vector_width_int (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_INT)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn native_vector_width_long (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_LONG)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(all(feature = "cl1_1", feature = "half"))]
    #[inline(always)]
    pub fn native_vector_width_half (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_HALF)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn native_vector_width_float (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_FLOAT)
    }

    /// Returns the native ISA vector width. The vector width is defined as the number of scalar elements that can be stored in the vector
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn native_vector_width_double (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_NATIVE_VECTOR_WIDTH_DOUBLE)
    }

    /// Is ```true``` if the device supports non-uniform work-groups, and ```false``` otherwise.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn non_uniform_work_group_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_NON_UNIFORM_WORK_GROUP_SUPPORT)?;
        Ok(v != 0)
    }

    /// Returns the highest fully backwards compatible OpenCL C version supported by the compiler for the device.
    #[docfg(feature = "cl1_1")]
    #[cfg_attr(feature = "cl3", deprecated)]
    #[inline(always)]
    pub fn opencl_c_version (&self) -> Result<String> {
        self.get_info_string(opencl_sys::CL_DEVICE_OPENCL_C_VERSION)
    }

    /// Returns the parent device to which this sub-device belongs. If device is a root-level device, a ```None``` value is returned.
    #[docfg(feature = "cl1_2")]
    #[inline]
    pub fn parent (&self) -> Result<Option<RawDevice>> {
        let v = self.get_info_bits::<cl_device_id>(opencl_sys::CL_DEVICE_PARENT_DEVICE)?;
        if let Some(v) = NonNull::new(v) {
            return Ok(Some(Self(v)))
        }

        Ok(None)
    }

    /// Returns the list of supported affinity domains for partitioning the device.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn partition_affinity_domain (&self) -> Result<Option<AffinityDomain>> {
        let v = self.get_info_bits::<opencl_sys::cl_device_affinity_domain>(opencl_sys::CL_DEVICE_PARTITION_PROPERTIES)?;

        Ok(match v {
            0 => None,
            _ => unsafe { Some(core::mem::transmute(v)) }
        })
    }

    /// Returns the properties argument specified in clCreateSubDevices if device is a sub-device.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn partition_type (&self) -> Result<Option<PartitionProperty>> {
        let v = self.get_info_array::<opencl_sys::cl_device_partition_property>(opencl_sys::CL_DEVICE_PARTITION_TYPE)?;
        Ok(PartitionProperty::from_slice(&v))
    } 

    /// Returns the maximum number of sub-devices that can be created when a device is partitioned. The value returned cannot exceed [max_compute_units](RawDevice::max_compute_units).
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn partition_max_sub_devices (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PARTITION_MAX_SUB_DEVICES)
    }

    /// Returns the list of partition types supported by device.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn partition_properties (&self) -> Result<Option<PartitionProperty>> {
        let v = self.get_info_array::<opencl_sys::cl_device_partition_property>(opencl_sys::CL_DEVICE_PARTITION_PROPERTIES)?;
        Ok(PartitionProperty::from_slice(&v))
    }

    /// Is ```true``` if the device supports pipes, and ```false``` otherwise. Devices that return ```true``` must also return ```true``` for [`generic_address_space_support`](RawDevice::generic_address_space_support).
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn pipe_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_PIPE_SUPPORT)?;
        Ok(v != 0)
    }

    /// The maximum number of reservations that can be active for a pipe per work-item in a kernel. A work-group reservation is counted as one reservation per work-item. The minimum value is 1 for devices supporting pipes, and must be 0 for devices that do not support pipes.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn pipe_max_active_reservations (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PIPE_MAX_ACTIVE_RESERVATIONS).map(NonZeroU32::new)
    }

    /// The maximum size of pipe packet in bytes. Support for pipes is required for an OpenCL 2.0, 2.1, or 2.2 device. The minimum value is 1024 bytes if the device supports pipes, and must be 0 for devices that do not support pipes.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn pipe_max_packet_size (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PIPE_MAX_PACKET_SIZE).map(NonZeroU32::new)
    }

    /// The platform associated with this device.
    #[inline(always)]
    pub fn platform (&self) -> Result<RawPlatform> {
        self.get_info_bits(CL_DEVICE_PLATFORM)
    }

    /// Is ```true``` if the devices preference is for the user to be responsible for synchronization, when sharing memory objects between OpenCL and other APIs such as DirectX, ```false``` if the device / implementation has a performant path for performing synchronization of memory object shared between OpenCL and other APIs such as DirectX.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn preferred_interop_user_sync (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_PREFERRED_INTEROP_USER_SYNC)?;
        Ok(v != 0)
    }

    /// Returns the value representing the preferred alignment in bytes for OpenCL 2.0 fine-grained SVM atomic types. This query can return 0 which indicates that the preferred alignment is aligned to the natural size of the type.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn preferred_platform_atomic_alignment (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_PLATFORM_ATOMIC_ALIGNMENT)
    }

    /// Returns the value representing the preferred alignment in bytes for OpenCL 2.0 atomic types to global memory. This query can return 0 which indicates that the preferred alignment is aligned to the natural size of the type.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn preferred_global_atomic_alignment (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_GLOBAL_ATOMIC_ALIGNMENT)
    }

    /// Returns the value representing the preferred alignment in bytes for OpenCL 2.0 atomic types to local memory. This query can return 0 which indicates that the preferred alignment is aligned to the natural size of the type.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn preferred_local_atomic_alignment (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_LOCAL_ATOMIC_ALIGNMENT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_char (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_CHAR)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_short (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_SHORT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_int (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_INT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_long (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_LONG)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_half (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_HALF)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_float (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_FLOAT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector. if the ```cl_khr_fp64``` extension is not supported, it must return 0.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn preferred_vector_width_double (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_VECTOR_WIDTH_DOUBLE)
    }

    /// Returns the preferred multiple of work-group size for the given device. This is a performance hint intended as a guide when specifying the local work size argument to clEnqueueNDRangeKernel.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn preferred_work_group_size_multiple (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PREFERRED_WORK_GROUP_SIZE_MULTIPLE)
    }

    /// Maximum size in bytes of the internal buffer that holds the output of printf calls from a kernel. The minimum value for the FULL profile is 1 MB.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn printf_buffer_size (&self) -> Result<NonZeroUsize> {
        self.get_info_bits(opencl_sys::CL_DEVICE_PRINTF_BUFFER_SIZE)
            .map(NonZeroUsize::new)
            .map(Option::unwrap)
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

    /// Describes the command-queue properties supported by the device.
    #[cfg_attr(feature = "cl2", deprecated(note = "see `queue_on_host_properties`"))]
    #[inline(always)]
    pub fn queue_properties (&self) -> Result<CommandQueueProperties> {
        let v = self.get_info_bits::<cl_command_queue_properties>(CL_DEVICE_QUEUE_PROPERTIES)?;
        Ok(CommandQueueProperties::from_bits(v))
    }

    /// Describes the on device command-queue properties supported by the device.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn queue_on_device_properties (&self) -> Result<CommandQueueProperties> {
        let v = self.get_info_bits::<cl_command_queue_properties>(opencl_sys::CL_DEVICE_QUEUE_ON_DEVICE_PROPERTIES)?;
        Ok(CommandQueueProperties::from_bits(v))
    }

    /// The maximum size of the device queue in bytes. The minimum value is 256 KB for the full profile and 64 KB for the embedded profile for devices supporting on-device queues, and must be 0 for devices that do not support on-device queues.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn queue_on_device_max_size (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_QUEUE_ON_DEVICE_MAX_SIZE).map(NonZeroU32::new)
    }

    /// The preferred size of the device queue, in bytes. Applications should use this size for the device queue to ensure good performance. The minimum value is 16 KB for devices supporting on-device queues, and must be 0 for devices that do not support on-device queues.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn queue_on_device_preferred_size (&self) -> Result<Option<NonZeroU32>> {
        self.get_info_bits(opencl_sys::CL_DEVICE_QUEUE_ON_DEVICE_PREFERRED_SIZE).map(NonZeroU32::new)
    }

    /// Describes the on host command-queue properties supported by the device.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn queue_on_host_properties (&self) -> Result<CommandQueueProperties> {
        let v = self.get_info_bits::<cl_command_queue_properties>(opencl_sys::CL_DEVICE_QUEUE_ON_HOST_PROPERTIES)?;
        Ok(CommandQueueProperties::from_bits(v))
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info_bits(opencl_sys::CL_DEVICE_REFERENCE_COUNT)
    }

    /// Describes single precision floating-point capability of the device.
    #[inline(always)]
    pub fn single_fp_config (&self) -> Result<FpConfig> {
        self.get_info_bits(CL_DEVICE_SINGLE_FP_CONFIG)
    }

    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn sub_group_independent_forward_progress (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_SUB_GROUP_INDEPENDENT_FORWARD_PROGRESS)?;
        Ok(v != 0)
    }    

    /// Describes the various shared virtual memory (SVM) memory allocation types the device supports.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn svm_capabilities (&self) -> Result<SvmCapability> {
        self.get_info_bits(opencl_sys::CL_DEVICE_SVM_CAPABILITIES)
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
    #[inline]
    pub fn version (&self) -> Result<Version> {
        let version = self.version_string()?;
        let section = version.split(' ').nth(1).ok_or(ErrorType::InvalidValue)?;
        Version::from_str(section).map_err(|_| ErrorType::InvalidValue.into())
    }

    /// Is ```true``` if the device supports work-group collective functions (e.g. work_group_broadcast, work_group_reduce and work_group_scan), and ```false``` otherwise.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn work_group_collective_functions_support (&self) -> Result<bool> {
        let v = self.get_info_bits::<cl_bool>(opencl_sys::CL_DEVICE_WORK_GROUP_COLLECTIVE_FUNCTIONS_SUPPORT)?;
        Ok(v != 0)
    }
    
    /// OpenCL software driver version string in the form _major_number_._minor_number_.
    #[inline(always)]
    pub fn driver_version_string (&self) -> Result<String> {
        self.get_info_string(CL_DRIVER_VERSION)
    }

    /// OpenCL software driver version
    #[inline(always)]
    pub fn driver_version (&self) -> Result<Version> {
        let driver = self.driver_version_string()?;
        Version::from_str(&driver).map_err(|_| ErrorType::InvalidValue.into())
    }

    /// Creates an array of sub-devices that each reference a non-intersecting set of compute units within in_device, according to the partition scheme given by properties. 
    /// The output sub-devices may be used in every way that the root (or parent) device can be used, including creating contexts, building programs, further calls to [`create_sub_devices`](RawDevice::create_sub_devices) and creating command-queues. 
    /// When a command-queue is created against a sub-device, the commands enqueued on the queue are executed only on the sub-device.
    #[docfg(feature = "cl1_2")]
    #[inline]
    pub fn create_sub_devices (&self, prop: PartitionProperty) -> Result<Vec<RawDevice>> {
        let prop = prop.to_bits();
        
        let mut len = 0;
        unsafe {
            tri!(opencl_sys::clCreateSubDevices(self.id(), prop.as_ptr(), 0, core::ptr::null_mut(), std::ptr::addr_of_mut!(len)))
        }

        let mut devices = Vec::with_capacity(len as usize);
        unsafe {
            tri!(opencl_sys::clCreateSubDevices(self.id(), prop.as_ptr(), len, devices.as_mut_ptr() as *mut _, core::ptr::null_mut()));
            devices.set_len(devices.capacity())
        }

        Ok(devices)
    }

    /// Replaces the default command queue on the device.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn set_default_command_queue (&self, ctx: crate::context::RawContext, queue: RawCommandQueue) -> Result<()> {
        unsafe {
            tri!(opencl_sys::clSetDefaultDeviceCommandQueue(ctx.id(), self.id(), queue.id()));
        }

        Ok(())
    }

    /// Query synchronized host and device timestamps.
    #[docfg(feature = "cl2_1")]
    #[inline]
    pub fn device_and_host_timer_nanos (&self) -> Result<[u64; 2]> {
        let mut device = 0;
        let mut host = 0;

        unsafe {
            tri!(clGetDeviceAndHostTimer(self.id(), std::ptr::addr_of_mut!(device), std::ptr::addr_of_mut!(host)))
        }

        Ok([device, host])
    }

    /// Query synchronized host and device timestamps.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn device_and_host_timer (&self) -> Result<(std::time::SystemTime, std::time::SystemTime)> {
        let [device, host] = self.device_and_host_timer_nanos()?;
        let device = std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_nanos(device)).unwrap();
        let host = std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_nanos(host)).unwrap();
        Ok((device, host))
    }

    /// Query the host clock.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn host_clock_nanos (&self) -> Result<u64> {
        let mut host = 0;
        unsafe {
            tri!(clGetHostTimer(self.id(), std::ptr::addr_of_mut!(host)))
        }

        Ok(host)
    }

    /// Query the host clock.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn host_clock (&self) -> Result<std::time::SystemTime> {
        let host = self.host_clock_nanos()?;
        Ok(std::time::UNIX_EPOCH + std::time::Duration::from_nanos(host))
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
    pub fn all () -> &'static [RawDevice] {
        &once_cell::sync::Lazy::force(&DEVICES)
    }

    #[inline(always)]
    pub fn first () -> Option<&'static RawDevice> {
        DEVICES.first()
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub unsafe fn retain (&self) -> Result<()> {
        tri!(clRetainDevice(self.id()));
        Ok(())
    }

    #[inline]
    fn get_info_string (&self, ty: cl_device_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            tri!(clGetDeviceInfo(self.id(), ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri!(clGetDeviceInfo(self.id(), ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));

            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn get_info_array<T: Copy> (&self, ty: cl_device_info) -> Result<Box<[T]>> {
        unsafe {
            let mut len = 0;
            tri!(clGetDeviceInfo(self.id(), ty, 0, core::ptr::null_mut(), &mut len));

            if len == 0 {
                return Ok(Box::new([]))
            }

            let mut result = Box::<[T]>::new_uninit_slice(len / core::mem::size_of::<T>());
            tri!(clGetDeviceInfo(self.id(), ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }

    #[inline]
    fn get_info_bits<T: Copy> (&self, ty: cl_device_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();

        unsafe {
            tri!(clGetDeviceInfo(self.id(), ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(value.assume_init())
        }
    }
}

impl Debug for RawDevice {
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

impl Clone for RawDevice {
    #[inline(always)]
    fn clone(&self) -> Self {
        #[cfg(feature = "cl1_2")]
        unsafe {
            tri_panic!(opencl_sys::clRetainDevice(self.id()))
        }

        Self(self.0)
    }
}

#[docfg(feature = "cl1_2")]
impl Drop for RawDevice {
    #[inline(always)]
    fn drop (&mut self) {
        unsafe {
            tri_panic!(opencl_sys::clReleaseDevice(self.id()));
        }
    }
}

unsafe impl Send for RawDevice {}
unsafe impl Sync for RawDevice {}

#[docfg(feature = "cl3")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct AtomicCapabilities {
    pub order: core::sync::atomic::Ordering,
    /// Support for memory ordering constraints that apply to a single work-item.
    pub work_item_scope: bool,
    pub scope: AtomicScope
}

#[docfg(feature = "cl3")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u64)]
pub enum AtomicScope {
    /// Support for memory ordering constraints that apply to all work-items in a work-group.
    WorkGroup = opencl_sys::CL_DEVICE_ATOMIC_SCOPE_WORK_GROUP,
    /// Support for memory ordering constraints that apply to all work-items executing on the device.
    Device = opencl_sys::CL_DEVICE_ATOMIC_SCOPE_DEVICE,
    /// Support for memory ordering constraints that apply to all work-items executing across all devices that can share SVM memory with each other and the host process.
    AllDevices = opencl_sys::CL_DEVICE_ATOMIC_SCOPE_ALL_DEVICES
}

#[cfg(feature = "cl3")]
impl AtomicCapabilities {
    pub fn from_bits (bits: opencl_sys::cl_device_atomic_capabilities) -> Option<Self> {
        let order;
        let scope;
        let work_item_scope = bits & opencl_sys::CL_DEVICE_ATOMIC_SCOPE_WORK_ITEM != 0;

        // ORDER
        if bits & opencl_sys::CL_DEVICE_ATOMIC_ORDER_SEQ_CST != 0 {
            order = core::sync::atomic::Ordering::SeqCst;
        }

        else if bits & opencl_sys::CL_DEVICE_ATOMIC_ORDER_ACQ_REL != 0 {
            order = core::sync::atomic::Ordering::AcqRel
        }

        else if bits & opencl_sys::CL_DEVICE_ATOMIC_ORDER_RELAXED != 0 {
            order = core::sync::atomic::Ordering::Relaxed
        }

        else {
            return None
        }

        // SCOPE
        if bits & opencl_sys::CL_DEVICE_ATOMIC_SCOPE_ALL_DEVICES != 0 {
            scope = AtomicScope::AllDevices
        }

        else if bits & opencl_sys::CL_DEVICE_ATOMIC_SCOPE_DEVICE != 0 {
            scope = AtomicScope::Device
        }

        else if bits & opencl_sys::CL_DEVICE_ATOMIC_SCOPE_WORK_GROUP != 0 {
            scope = AtomicScope::WorkGroup
        }

        else {
            return None;
        }

        Some(Self { order, work_item_scope, scope  })
    }
}

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
        /// Denorms are supported
        const DENORM = CL_FP_DENORM;
        /// INF and quiet NaNs are supported
        const INF_NAN = CL_FP_INF_NAN;
        /// Round to nearest even rounding mode supported
        const ROUND_TO_NEAREST = CL_FP_ROUND_TO_NEAREST;
        /// Round to zero rounding mode supported
        const ROUND_TO_ZERO = CL_FP_ROUND_TO_ZERO;
        /// Round to positive and negative infinity rounding modes supported
        const ROUND_TO_INF = CL_FP_ROUND_TO_INF;
        /// IEEE754-2008 fused multiply-add is supported
        const FMA = CL_FP_FMA;
        /// Divide and sqrt are correctly rounded as defined by the IEEE754 specification
        const CORRECTLY_ROUNDED_DIVIDE_SQRT = CL_FP_CORRECTLY_ROUNDED_DIVIDE_SQRT;
        /// Basic floating-point operations (such as addition, subtraction, multiplication) are implemented in software
        const SOFT_FLOAT = CL_FP_SOFT_FLOAT;
    }

    /// Describes the execution capabilities of the device
    #[repr(transparent)]
    pub struct ExecCapabilities : cl_device_exec_capabilities {
        const KERNEL = CL_EXEC_KERNEL;
        const NATIVE_KERNEL = CL_EXEC_NATIVE_KERNEL;
    }
}

/// Type of local memory supported. This can be set to [```Self::Local```] implying dedicated local memory storage such as SRAM, or [```Self::Global```].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum LocalMemType {
    Local = CL_LOCAL,
    Global = CL_GLOBAL
}

#[docfg(feature = "cl1_2")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PartitionProperty {
    /// Split the aggregate device into as many smaller aggregate devices as can be created, each containing n compute units. The value n is passed as the value accompanying this property. If n does not divide evenly into [`max_compute_units`](RawDevice::max_compute_units), then the remaining compute units are not used.
    Equally (u32),
    /// This property is followed by a list of compute unit. For each non-zero count m in the list, a sub-device is created with m compute units in it. The number of non-zero count entries in the list may not exceed [`partition_max_sub_devices`](RawDevice::partition_max_sub_devices). The total number of compute units specified may not exceed [max_compute_units](RawDevice::max_compute_units).
    Counts (Vec<NonZeroU32>),
    /// Split the device into smaller aggregate devices containing one or more compute units that all share part of a cache hierarchy.
    AffinityDomain (AffinityDomain)
}

#[cfg(feature = "cl1_2")]
impl PartitionProperty {
    pub fn from_slice (bits: &[opencl_sys::cl_device_partition_property]) -> Option<Self> {
        if bits.len() == 0 {
            return None;
        }

        match unsafe { *bits.get_unchecked(0) } {
            0 => None,
            opencl_sys::CL_DEVICE_PARTITION_EQUALLY => Some(Self::Equally(bits[1] as u32)),
            opencl_sys::CL_DEVICE_PARTITION_BY_AFFINITY_DOMAIN => Some(Self::AffinityDomain(unsafe { core::mem::transmute(bits[1] as u64) })),
            opencl_sys::CL_DEVICE_PARTITION_BY_COUNTS => {
                let mut result = Vec::with_capacity(bits.len());

                for i in 1..bits.len() {
                    const MAX_COUNT : isize = u32::MAX as isize;

                    match bits[i] {
                        #[allow(unreachable_patterns)]
                        0 | opencl_sys::CL_DEVICE_PARTITION_BY_COUNTS_LIST_END => break,
                        v @ 1..=MAX_COUNT => unsafe { result.push(NonZeroU32::new_unchecked(v as u32)) }
                        _ => return None
                    }
                }

                Some(Self::Counts(result))
            },

            other => panic!("Unknow partition property '{other}'")
        }
    }

    pub fn to_bits (&self) -> Box<[opencl_sys::cl_device_partition_property]> {
        match self {
            Self::Equally(n) => Box::new([opencl_sys::CL_DEVICE_PARTITION_EQUALLY, opencl_sys::cl_device_partition_property::try_from(*n).unwrap(), 0]) as Box<_>,
            Self::AffinityDomain(x) => Box::new([opencl_sys::CL_DEVICE_PARTITION_BY_AFFINITY_DOMAIN, opencl_sys::cl_device_partition_property::try_from(*x as u64).unwrap(), 0]) as Box<_>,
            Self::Counts(x) => {
                let mut result = Box::new_uninit_slice(2 + x.len());
                
                unsafe {
                    result[0].write(opencl_sys::CL_DEVICE_PARTITION_BY_COUNTS);
                    
                    for i in 0..x.len() {
                        result[1 + i].write(opencl_sys::cl_device_partition_property::try_from(x[i].get()).unwrap());
                    }

                    result.last_mut().unwrap_unchecked().write(opencl_sys::CL_DEVICE_PARTITION_BY_COUNTS_LIST_END);
                    result.assume_init()
                }
            }
        }
    }
}

#[docfg(feature = "cl1_2")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
#[non_exhaustive]
pub enum AffinityDomain {
    /// Split the device into sub-devices comprised of compute units that share a NUMA node.
    Numa = opencl_sys::CL_DEVICE_AFFINITY_DOMAIN_NUMA,
    /// Split the device into sub-devices comprised of compute units that share a level 4 data cache.
    L4Cache = opencl_sys::CL_DEVICE_AFFINITY_DOMAIN_L4_CACHE,
    /// Split the device into sub-devices comprised of compute units that share a level 3 data cache.
    L3Cache = opencl_sys::CL_DEVICE_AFFINITY_DOMAIN_L3_CACHE,
    /// Split the device into sub-devices comprised of compute units that share a level 2 data cache.
    L2Cache = opencl_sys::CL_DEVICE_AFFINITY_DOMAIN_L2_CACHE,
    /// Split the device into sub-devices comprised of compute units that share a level 1 data cache.
    L1Cache = opencl_sys::CL_DEVICE_AFFINITY_DOMAIN_L1_CACHE,
    /// Split the device along the next partitionable affinity domain. The implementation shall find the first level along which the device or sub-device may be further subdivided in the order NUMA, L4, L3, L2, L1, and partition the device into sub-devices comprised of compute units that share memory subsystems at this level.
    NextPartitionable = opencl_sys::CL_DEVICE_AFFINITY_DOMAIN_NEXT_PARTITIONABLE
}

#[docfg(feature = "cl3")]
#[repr(u64)]
#[non_exhaustive]
pub enum DeviceEnqueueCapabilities {
    /// Device supports device-side enqueue and on-device queues.
    Supported = opencl_sys::CL_DEVICE_QUEUE_SUPPORTED,
    /// Device supports a replaceable default on-device queue.
    ReplaceableDefault = opencl_sys::CL_DEVICE_QUEUE_REPLACEABLE_DEFAULT
}

#[cfg(feature = "cl3")]
impl DeviceEnqueueCapabilities {
    pub fn from_bits (bits: opencl_sys::cl_device_device_enqueue_capabilities) -> Option<Self> {
        if bits & opencl_sys::CL_DEVICE_QUEUE_REPLACEABLE_DEFAULT != 0 {
            return Some(Self::ReplaceableDefault);
        }

        if bits & opencl_sys::CL_DEVICE_QUEUE_SUPPORTED != 0 {
            return Some(Self::Supported);
        }

        None
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct SvmCapability: cl_device_svm_capabilities {
        /// Support for coarse-grain buffer sharing using clSVMAlloc. Memory consistency is guaranteed at synchronization points and the host must use calls to clEnqueueMapBuffer and clEnqueueUnmapMemObject.
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