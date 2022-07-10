use crate::prelude::*;
use crate::core::*;

#[derive(Clone)]
#[non_exhaustive]
pub struct ImageDesc {
    /// Describes the image type.
    pub ty: MemObjectType,
    /// The width of the image in pixels.
    pub width: usize,
    /// The height of the image in pixels.
    pub height: usize,
    /// The depth of the image in pixels. This is only used if the image is a 3D image and must be a value ≥ 1 and ≤ CL_DEVICE_IMAGE3D_MAX_DEPTH.
    pub depth: usize,
    /// The number of images in the image array. This is only used if the image is a 1D or 2D image array. The values for image_array_size, if specified, must be a value ≥ 1 and ≤ CL_DEVICE_IMAGE_MAX_ARRAY_SIZE.\
    /// Note that reading and writing 2D image arrays from a kernel with image_array_size = 1 may be lower performance than 2D images.
    pub array_size: usize,
    /// The scan-line pitch in bytes. This must be 0 if host_ptr is NULL and can be either 0 or ≥ image_width * size of element in bytes if host_ptr is not NULL. 
    /// If host_ptr is not NULL and image_row_pitch = 0, image_row_pitch is calculated as image_width * size of element in bytes. If image_row_pitch is not 0, it must be a multiple of the image element size in bytes.
    /// For a 2D image created from a buffer, the pitch specified (or computed if pitch specified is 0) must be a multiple of the maximum of the CL_DEVICE_IMAGE_PITCH_ALIGNMENT value for all devices in the context associated with image_desc->mem_object and that support images.
    pub row_pitch: usize,
    /// The size in bytes of each 2D slice in the 3D image or the size in bytes of each image in a 1D or 2D image array. 
    /// This must be 0 if host_ptr is NULL. If host_ptr is not NULL, image_slice_pitch can be either 0 or ≥ image_row_pitch * image_height for a 2D image array or 3D image and can be either 0 or ≥ image_row_pitch for a 1D image array.
    /// If host_ptr is not NULL and image_slice_pitch = 0, image_slice_pitch is calculated as image_row_pitch * image_height for a 2D image array or 3D image and image_row_pitch for a 1D image array. 
    /// If image_slice_pitch is not 0, it must be a multiple of the image_row_pitch.
    pub slice_pitch: usize,
    pub mem_object: Option<MemObject>
}

pub struct Builder (ImageDesc, Device);

impl Builder {
    pub const fn new (dev: Device, ty: MemObjectType, width: usize, height: usize) -> Self {
        let v = ImageDesc {
            ty, width, height,
            depth: 0,
            array_size: 0,
            row_pitch: 0,
            slice_pitch: 0,
            mem_object: None,
        };

        Self(v, dev)
    }

    pub fn set_depth (&mut self, v: usize) -> Result<&mut Self> {
        if self.0.ty != MemObjectType::Image3D {
            return Err(Error::InvalidMemObject)
        }

        todo!()
    }

    #[inline(always)]
    pub fn build (self) -> ImageDesc {
        self.0
    }
}