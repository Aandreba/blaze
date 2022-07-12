use std::{ops::{DerefMut, Deref}, io::Write};

use image::{Rgba, imageops};
use rscl::{core::*, context::{SimpleContext, Global}, buffer::{Buffer, flags::MemAccess}, event::WaitList, prelude::Event, memobj::MapBoxExt, image::Image2D};
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
    println!("{}", core::mem::size_of::<Device>());
    println!("{}", core::mem::size_of::<Option<Device>>());

    println!("{}", core::mem::size_of::<bool>());
    println!("{}", core::mem::size_of::<Option<bool>>());

    let dev = Device::first().unwrap();
    println!("{:?}", dev.device_and_host_timer());
    Ok(())
}

#[test]
fn flag () {
    let mut img = Image2D::<Rgba<u8>>::from_file("tests/test.png", MemAccess::default(), false).unwrap();
    let mut map = img.map_mut((32..64, 32..75), WaitList::EMPTY).unwrap().wait().unwrap();
    imageops::rotate180_in_place(&mut map);

    drop(map);
    img.read_all(WaitList::EMPTY).unwrap().wait().unwrap().save("tests/test_map.png").unwrap();
}