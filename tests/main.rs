use std::{ops::{DerefMut, Deref}, io::Write, time::Duration, sync::Arc};
use image::{Rgba, imageops};
use rscl::{core::*, context::{SimpleContext}, buffer::{Buffer, flags::MemAccess, BufferRect2D, rect::Rect2D}, event::{WaitList, FlagEvent}, prelude::Event, image::{Image2D, Sampler, SamplerProperties, AddressingMode}};
use rscl_proc::global_context;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn program () -> Result<()> {
    let sampler = Sampler::new(SamplerProperties::new(false, AddressingMode::default(), rscl::image::FilterMode::Linear))?;
    println!("{:?}", sampler.properties());

    Ok(())
}

#[test]
fn flag () {
    /*
        [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]
    */
    let rect = Rect2D::<u16>::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3).unwrap(); // 3 x 3
    let buf = BufferRect2D::new(&rect, MemAccess::default(), false).unwrap();

    /*
        [
            [2, 3],
            [5, 6],
            [8, 9]
        ]
    */
    let read = buf.read((1.., ..), WaitList::EMPTY).unwrap().wait_unwrap();
    println!("{read:?}")
    //println!("{:?}", read.as_slice().into_iter().map(|x| format!("{:016b}", x >> 8)).collect::<Vec<_>>())
}