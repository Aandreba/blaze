use std::{sync::Arc, thread, time::Duration, io::Write};
use rscl::{core::*, context::{SimpleContext, ContextProperties}, buffer::{flags::MemAccess, Buffer, events::ReadBuffer}, event::{WaitList, FlagEvent}, prelude::{EventExt, Event}};
use rscl_proc::{global_context};

fn log (s: &str) {
    let mut stdout = std::io::stdout().lock();
    stdout.write_fmt(format_args!("{s}\n")).unwrap()
}

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::with_loger(Device::first().unwrap(), ContextProperties::default(), CommandQueueProperties::new(true, true), log);

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn program () -> Result<()> {
    println!("{:?}", Device::first().unwrap().queue_on_host_properties());
    Ok(())
}

#[test]
fn flag () -> Result<()> {
    let ctx = SimpleContext::with_loger(Device::first().unwrap(), ContextProperties::default(), CommandQueueProperties::new(true, true), log)?;

    let buffer = Arc::new(Buffer::new_in(ctx.clone(), &[1, 2, 3, 4, 5, 6], MemAccess::default(), false)?);
    let flag = FlagEvent::new_in(&ctx)?;

    let one = buffer.clone().read_owned(..3, [flag.to_raw()])?;
    let two = buffer.read_owned(3.., WaitList::EMPTY)?;

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        flag.set_complete(None).unwrap()
    });

    let join = ReadBuffer::join_in(&ctx, [one, two])?;
    let result = join.wait()?;
    println!("{result:?}");

    Ok(())
}