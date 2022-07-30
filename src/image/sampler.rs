use std::{ptr::{NonNull, addr_of_mut}, ffi::c_void, mem::MaybeUninit};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use opencl_sys::*;
use crate::prelude::{*};

#[repr(transparent)]
pub struct Sampler (NonNull<c_void>);

impl Sampler {
    #[inline(always)]
    pub fn new (props: SamplerProperties) -> Result<Self> {
        Self::new_in(&Global, props)
    }

    #[cfg(not(featue = "cl2"))]
    pub fn new_in (ctx: &RawContext, props: SamplerProperties) -> Result<Self> {
        let mut err = 0;
        let id;

        #[allow(deprecated)]
        unsafe {
            id = clCreateSampler(ctx.id(), props.normalized_coords as cl_bool, props.addressing_mode as cl_addressing_mode, props.filter_mode as cl_filter_mode, addr_of_mut!(err));
            if err != 0 { return Err(Error::from(err)) }
            Ok(Self::from_id(id).unwrap())
        }
    }

    #[cfg(featue = "cl2")]
    pub fn new_in (ctx: &RawContext, props: SamplerProperties) -> Result<Self> {
        let mut err = 0;
        let id;

        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(feature = "strict")] {
                    let props = props.to_bits();
                    id = clCreateSamplerWithProperties(ctx.id(), props.as_ptr(), addr_of_mut!(err))
                } else {
                    #[allow(deprecated)]
                    if ctx.greatest_common_version()? >= Version::CL2 {
                        let props = props.to_bits();
                        id = clCreateSamplerWithProperties(ctx.id(), props.as_ptr(), addr_of_mut!(err));
                    } else {
                        id = clCreateSampler(ctx.id(), props.normalized_coords as cl_bool, props.addressing_mode as cl_addressing_mode, props.filter_mode as cl_filter_mode, addr_of_mut!(err))
                    }
                }
            }

            if err != 0 { return Err(Error::from(err)) }
            Ok(Self::from_id(id).unwrap())
        }
    }

    #[inline(always)]
    pub const unsafe fn from_id (id: cl_sampler) -> Option<Self> {
        NonNull::new(id).map(Self)
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_sampler) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_sampler {
        self.0.as_ptr()
    }
}

impl Sampler {
    /// Return the sampler reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_SAMPLER_REFERENCE_COUNT)
    }

    /// Return the context specified when the sampler is created.
    #[inline(always)]
    pub fn context (&self) -> Result<RawContext> {
        self.get_info(CL_SAMPLER_CONTEXT)
    }

    /// Return the normalized coords value associated with sampler.
    #[inline(always)]
    pub fn normalized_coords (&self) -> Result<bool> {
        let v = self.get_info::<cl_bool>(CL_SAMPLER_NORMALIZED_COORDS)?;
        Ok(v != 0)
    }

    #[inline(always)]
    pub fn addressing_mode (&self) -> Result<AddressingMode> {
        self.get_info(CL_SAMPLER_ADDRESSING_MODE)
    }

    #[inline(always)]
    pub fn filter_mode (&self) -> Result<FilterMode> {
        self.get_info(CL_SAMPLER_FILTER_MODE)
    }

    #[cfg(feature = "cl3")]
    #[inline(always)]
    pub fn properties (&self) -> Result<SamplerProperties> {
        let v = self.get_info_array::<cl_sampler_properties>(CL_SAMPLER_PROPERTIES)?;
        Ok(SamplerProperties::from_bits(&v))
    }

    #[cfg(not(feature = "cl3"))]
    #[inline]
    pub fn properties (&self) -> Result<SamplerProperties> {
        let normalized_coords = self.normalized_coords()?;
        let addressing_mode = self.addressing_mode()?;
        let filter_mode = self.filter_mode()?;
        Ok(SamplerProperties::new(normalized_coords, addressing_mode, filter_mode))
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_sampler_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();

        unsafe {
            tri!(clGetSamplerInfo(self.id(), ty, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }

    #[inline]
    fn get_info_array<T: Copy> (&self, ty: cl_sampler_info) -> Result<Box<[T]>> {
        let mut size = 0;
        unsafe {
            tri!(clGetSamplerInfo(self.id(), ty, 0, core::ptr::null_mut(), addr_of_mut!(size)))
        }

        let len = size / core::mem::size_of::<T>();
        let mut result = Box::<[T]>::new_uninit_slice(len);

        unsafe {
            tri!(clGetSamplerInfo(self.id(), ty, size, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl Clone for Sampler {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainSampler(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for Sampler {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseSampler(self.id()))
        }
    }
}

unsafe impl Send for Sampler {}
unsafe impl Sync for Sampler {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SamplerProperties {
    /// A boolean value that specifies whether the image coordinates specified are normalized or not.
    pub normalized_coords: bool,
    /// Specifies how out-of-range image coordinates are handled when reading from an image.
    pub addressing_mode: AddressingMode,
    /// Specifies the type of filter that is applied when reading an image.
    pub filter_mode: FilterMode
}

impl SamplerProperties {
    const LEN : usize = 3 * 2 + 1;

    #[inline(always)]
    pub const fn new (normalized_coords: bool, addressing_node: AddressingMode, filter_mode: FilterMode) -> Self {
        Self { normalized_coords, addressing_mode: addressing_node, filter_mode }
    }

    #[inline]
    pub fn from_bits (bits: &[cl_sampler_properties]) -> Self {
        let mut result = Self::default();
        if bits.len() == 0 { return result; }

        for i in (0..Self::LEN).step_by(2) {
            if bits.len() == i { return result; }

            match bits[i] as u32 {
                CL_SAMPLER_NORMALIZED_COORDS => result.normalized_coords = bits[i + 1] != 0,
                CL_SAMPLER_ADDRESSING_MODE => result.addressing_mode = AddressingMode::try_from_primitive(bits[i + 1] as u32).unwrap(),
                CL_SAMPLER_FILTER_MODE => result.filter_mode = FilterMode::try_from_primitive(bits[i + 1] as u32).unwrap(),
                0 => return result,
                _ => unimplemented!()
            }
        }

        todo!()
    }

    #[inline(always)]
    pub fn to_bits (&self) -> [cl_sampler_properties; Self::LEN] {
        [
            CL_SAMPLER_NORMALIZED_COORDS as cl_sampler_properties, self.normalized_coords as cl_sampler_properties,
            CL_SAMPLER_ADDRESSING_MODE as cl_sampler_properties, self.addressing_mode as cl_sampler_properties,
            CL_SAMPLER_FILTER_MODE as cl_sampler_properties, self.filter_mode as cl_sampler_properties,
            0
        ]
    }
}

impl Default for SamplerProperties {
    #[inline(always)]
    fn default() -> Self {
        Self { 
            normalized_coords: true,
            addressing_mode: Default::default(),
            filter_mode: Default::default()
        }
    }
}

/// Specifies how out-of-range image coordinates are handled when reading from an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[non_exhaustive]
#[repr(u32)]
pub enum AddressingMode {
    /// Behavior is undefined for out-of-range image coordinates.
    None = CL_ADDRESS_NONE,
    /// Out-of-range image coordinates are clamped to the edge of the image.
    ClampToEdge = CL_ADDRESS_CLAMP_TO_EDGE,
    /// Out-of-range image coordinates are assigned a border color value.
    Clamp = CL_ADDRESS_CLAMP,
    /// Out-of-range image coordinates read from the image as-if the image data were replicated in all dimensions.
    Repeat = CL_ADDRESS_REPEAT,
    /// Out-of-range image coordinates read from the image as-if the image data were replicated in all dimensions, mirroring the image contents at the edge of each replication.
    MirroredRepeat = CL_ADDRESS_MIRRORED_REPEAT
}

impl Default for AddressingMode {
    #[inline(always)]
    fn default() -> Self {
        Self::Clamp
    }
}

/// Specifies the type of filter that is applied when reading an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[non_exhaustive]
#[repr(u32)]
pub enum FilterMode {
    /// Returns the image element nearest to the image coordinate.
    Nearest = CL_FILTER_NEAREST,
    /// Returns a weighted average of the four image elements nearest to the image coordinate.
    Linear = CL_FILTER_LINEAR
}

impl Default for FilterMode {
    #[inline(always)]
    fn default() -> Self {
        Self::Nearest
    }
}