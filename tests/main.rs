use std::{ops::{DerefMut, Deref}, io::Write, time::Duration};
use image::{Rgba, imageops};
use rscl::{core::*, context::{SimpleContext}, buffer::{Buffer, flags::MemAccess}, event::{WaitList, FlagEvent}, prelude::Event, memobj::MapBoxExt, image::{Image2D, Sampler, SamplerProperties, AddressingMode}};
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
    let mut img = Image2D::<Rgba<u8>>::from_file("tests/test.png", MemAccess::default(), false).unwrap();
    let map_evt = img.map_mut((32..64, 32..75), WaitList::EMPTY).unwrap(); 
    
    map_evt.as_raw().on_complete(|evt, _| {
        let prof = evt.profiling_time().unwrap();
        println!("{:?}", prof.duration())
    }).unwrap();

    let mut map = map_evt.wait().unwrap();
    imageops::rotate180_in_place(&mut map);

    drop(map);
    img.read_all(WaitList::EMPTY).unwrap().wait().unwrap().save("tests/test_map.png").unwrap();
}