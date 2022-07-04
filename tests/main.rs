use rscl::{core::*, buffer::{Buffer, MemObject, WriteBuffer}, event::{FlagEvent, Event}, context::SimpleContext};
use rscl_proc::global_context;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn read_after_free () -> Result<()> {
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], false)?;
    let event = FlagEvent::new()?;

    let read = buffer.read_all(&event)?;
    drop(buffer);
    event.set_complete(None)?;
    let data = read.wait()?;

    println!("{data:?}");
    Ok(())
}

#[test]
fn write_after_free () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], false)?;
    let event = FlagEvent::new()?;

    println!("{}", buffer.reference_count()?);
    let write = buffer.write(vec![1, 2, 3], 0, &event)?;
    println!("{}", buffer.reference_count()?);
    
    drop(buffer);
    event.set_complete(None)?;
    write.wait()?;

    Ok(())
}

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn program () -> Result<()> {
    let dev = Device::first().unwrap();
    println!("{:?}", dev.extensions()?);
    Ok(())
}