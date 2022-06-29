use rscl::{core::*, buffer::{Buffer, flags::MemFlags}, event::{FlagEvent, Event}, context::SimpleContext};
use rscl_proc::global_context;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn read_after_free () -> Result<()> {
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemFlags::default())?;
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
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemFlags::default())?;
    let event = FlagEvent::new()?;

    let write = buffer.write(vec![1, 2, 3], 0, &event)?;
    println!("{}", buffer.reference_count()?);
    drop(buffer);
    event.set_complete(None)?;
    write.wait()?;

    Ok(())
}