#![feature(new_uninit)]

use std::mem::MaybeUninit;
use rscl::{prelude::Event, buffer::{RawBuffer, flags::{MemFlags, HostPtr}}};
use image::{Rgb};
use rscl::{context::SimpleContext, prelude::Result, image::Image2D, buffer::{flags::MemAccess, rect::Rect2D, Buffer}, event::WaitList};
use rscl_proc::{global_context, rscl};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

static CODE : &str = "
__constant sampler_t sampler = CLK_NORMALIZED_COORDS_FALSE | CLK_ADDRESS_CLAMP_TO_EDGE | CLK_FILTER_NEAREST;
 
__kernel void gaussian_blur(
        __read_only image2d_t image,
        __constant float * mask,
        __global float * blurredImage,
        __private int maskSize
    ) {
 
    const int2 pos = {get_global_id(0), get_global_id(1)};
 
    // Collect neighbor values and multiply with Gaussian
    float sum = 0.0f;
    for(int a = -maskSize; a < maskSize+1; a++) {
        for(int b = -maskSize; b < maskSize+1; b++) {
            sum += mask[a+maskSize+(b+maskSize)*(maskSize*2+1)]
                *read_imagef(image, sampler, pos + (int2)(a,b)).x;
        }
    }
 
    blurredImage[pos.x+pos.y*get_global_size(0)] = sum;
}
";

#[rscl(GaussBlur)]
#[link(CODE)]
extern {
    fn gaussian_blur (image: image2d, mask: *const f32, blurred: *mut MaybeUninit<f32>, size: i32);
}

#[test]
fn gauss () -> Result<()> {
    let gauss = GaussBlur::new(None)?;

    let image = Image2D::<Rgb<f32>>::from_file("tests/test2.jpg", MemAccess::READ_ONLY, false)?;
    let mut result = Buffer::<f32>::new_uninit(image.width()? * image.height()?, MemAccess::WRITE_ONLY, false)?;
    
    let (mask_size, mask) = create_blur_mask(10.);
    let mask = Buffer::new(mask.as_slice(), MemAccess::READ_ONLY, false)?;

    let evt = unsafe { gauss.gaussian_blur(&image, &mask, &mut result, mask_size, [image.width()?, image.height()?, 1], None, WaitList::EMPTY)? };
    let _ = evt.wait()?;

    let result = unsafe { result.assume_init().read_all(WaitList::EMPTY)?.wait()? };
    println!("{:?}", &result[..10]);
    
    Ok(())
}

fn create_blur_mask (sigma: f32) -> (i32, Rect2D<f32>) {
    let mask_size = f32::ceil(3.0 * sigma) as i32;
    let len = mask_size * 2 + 1;

    let mut mask = Rect2D::<f32>::new_uninit(len as usize, len as usize).unwrap();
    let mut sum = 0.;

    for a in -mask_size..=mask_size {
        let norm_a = (a + mask_size) as usize;
        for b in -mask_size..=mask_size {
            let norm_b = (b + mask_size) as usize;
            let temp = f32::hypot(a as f32, b as f32);
            let temp = f32::exp(-temp / (2. * sigma * sigma));

            sum += temp;
            mask[(norm_a, norm_b)].write(temp);
        }
    }

    let mut mask = unsafe { mask.assume_init() };
    mask.rows_iter_mut().flat_map(|x| x.iter_mut()).for_each(|x| *x /= sum);
    (mask_size, mask)
}