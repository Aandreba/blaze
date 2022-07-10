use opencl_sys::{cl_image_desc, cl_mem_object_type};
use crate::prelude::*;

#[derive(Clone)]
#[non_exhaustive]
pub struct ImageDesc {
    /// Describes the image type.
    pub ty: MemObjectType,
    /// The width of the image in pixels.
    pub width: usize,
    /// The height of the image in pixels.
    pub height: usize,
    /// The depth of the image in pixels. This is only used if the image is a 3D image and must be a value ≥ 1 and ≤ [`Device::image3d_max_depth`].
    pub depth: usize,
    /// The number of images in the image array. This is only used if the image is a 1D or 2D image array. The values for image_array_size, if specified, must be a value ≥ 1 and ≤ [`Device::image_max_array_size`].\
    /// Note that reading and writing 2D image arrays from a kernel with image_array_size = 1 may be lower performance than 2D images.
    pub array_size: usize,
    /// The scan-line pitch in bytes. This must be 0 if host_ptr is NULL and can be either 0 or ≥ image_width * size of element in bytes if host_ptr is not NULL. 
    /// If host_ptr is not NULL and image_row_pitch = 0, image_row_pitch is calculated as image_width * size of element in bytes. If image_row_pitch is not 0, it must be a multiple of the image element size in bytes.
    /// For a 2D image created from a buffer, the pitch specified (or computed if pitch specified is 0) must be a multiple of the maximum of the [`Device::image_pitch_alignment`] value for all devices in the context associated with image_desc->mem_object and that support images.
    pub row_pitch: usize,
    /// The size in bytes of each 2D slice in the 3D image or the size in bytes of each image in a 1D or 2D image array. 
    /// This must be 0 if host_ptr is NULL. If host_ptr is not NULL, image_slice_pitch can be either 0 or ≥ image_row_pitch * image_height for a 2D image array or 3D image and can be either 0 or ≥ image_row_pitch for a 1D image array.
    /// If host_ptr is not NULL and image_slice_pitch = 0, image_slice_pitch is calculated as image_row_pitch * image_height for a 2D image array or 3D image and image_row_pitch for a 1D image array. 
    /// If image_slice_pitch is not 0, it must be a multiple of the image_row_pitch.
    pub slice_pitch: usize,
    /// May refer to a valid buffer or image memory object. mem_object can be a buffer memory object if image_type is CL_MEM_OBJECT_IMAGE1D_BUFFER or CL_MEM_OBJECT_IMAGE2D.
    /// mem_object can be an image object if image_type is CL_MEM_OBJECT_IMAGE2D. Otherwise it must be NULL. The image pixels are taken from the memory objects data store. 
    /// When the contents of the specified memory objects data store are modified, those changes are reflected in the contents of the image object and vice-versa at corresponding synchronization points.
    pub mem_object: Option<MemObject>
}

impl ImageDesc {
    #[inline(always)]
    pub const fn new (ty: MemObjectType, width: usize, height: usize) -> Self {
        Self { 
            ty, width, height,
            depth: 0, 
            array_size: 0, 
            row_pitch: 0, 
            slice_pitch: 0,
            mem_object: None
        }
    }

    #[inline(always)]
    pub const fn to_raw (&self) -> cl_image_desc {
        let buffer = match self.mem_object {
            Some(ref mem_object) => mem_object.id(),
            None => core::ptr::null_mut()
        };

        cl_image_desc {
            image_type: self.ty as cl_mem_object_type,
            image_width: self.width,
            image_height: self.height,
            image_depth: self.depth,
            image_array_size: self.array_size,
            image_row_pitch: self.row_pitch,
            image_slice_pitch: self.slice_pitch,
            num_mip_levels: 0,
            num_samples: 0,
            buffer
        }
    }
}