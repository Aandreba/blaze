use std::{ops::{DerefMut, Deref}, io::Write, time::Duration, sync::Arc};
use image::{Rgba, imageops};
use rscl::{core::*, context::{SimpleContext}, buffer::{Buffer, flags::MemAccess}, event::{WaitList, FlagEvent}, prelude::Event, image::{Image2D, Sampler, SamplerProperties, AddressingMode}};
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
    let buf = Arc::new(Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false).unwrap());
    let map = buf.map_owned(0..2, WaitList::EMPTY).unwrap().wait_unwrap();

    for i in map.into_iter() {
        println!("{i}")
    }
}