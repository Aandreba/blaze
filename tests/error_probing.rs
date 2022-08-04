use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    let read = buffer.read(.., EMPTY)?;
    let raw = read.to_raw();

    read.wait()?;
    raw.as_raw().on_run(move |_, _| println!("Hello"))?;
    raw.as_raw().on_complete(move |_, _| println!("I'm done"))?;

    Ok(())
}