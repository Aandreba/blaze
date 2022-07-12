use rscl::{core::*, context::{SimpleContext, Global}, buffer::{Buffer, flags::MemAccess}, event::WaitList, prelude::Event};
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
    println!("{:?}", dev.atomic_memory_capabilities());
    Ok(())
}

#[test]
fn flag () {
    let mut buf = Buffer::new(&[1u64, 2, 3, 4, u64::MAX], MemAccess::default(), false).unwrap();
    let map = buf.map(1.., WaitList::EMPTY).unwrap().wait().unwrap();

    for i in map.into_iter() {
        println!("{i}")
    }

    println!("{buf:?}")
}