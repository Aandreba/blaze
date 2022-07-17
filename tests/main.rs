use std::sync::Arc;

use rscl::{core::*, context::{SimpleContext}, buffer::{flags::MemAccess, Buffer, events::ReadBuffer}, event::{WaitList, FlagEvent}, prelude::{Global, Context, EventExt, Event}};
use rscl_proc::{global_context};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn program () -> Result<()> {
    let version = Global.next_queue().size()?;
    println!("{version:?}");
    Ok(())
}

#[test]
fn flag () -> Result<()> {
    let buffer = Arc::new(Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?);
    let one = buffer.clone().read_owned(..2, WaitList::EMPTY)?;
    let two = buffer.read_owned(2.., WaitList::EMPTY)?;
    
    println!("{}", core::mem::size_of_val(&one));
    one.boxed_local().wait();

    /*let flag = FlagEvent::new()?;
    let three = flag.to_raw().map(|_| vec![7, 8, 9])?;
    let iter = [one.boxed(), two.boxed(), three.boxed()];

    let join = Box::<dyn Event<Output = Vec<i32>>>::join(iter);
    let out = join.wait_async()?;
    println!("{out:?}");*/

    Ok(())
}