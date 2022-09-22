#![feature(nonzero_min_max)]
use blaze_rs::{prelude::*};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    Ok(())
}

#[cfg(feature = "cl1_2")]
#[test]
fn test () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    //let slice = buffer.slice_mut(..)?;
    
    let [left, right] = scope(|s| {
        let left = buffer.read(s, ..2, None)?;
        let right = buffer.read(s, 2.., None)?;
        println!("{left:?}");
        let v = Event::join_all_sized_blocking([left, right]);
        println!("Done!");
        return v;
    })?;
    
    println!("{left:?}, {right:?}");
    Ok(())
}

// TODO Rethink system for non-blocking events
#[cfg(feature = "futures")]
#[tokio::test]
async fn async_test () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;

    let mut event = blaze_rs::context::local_scope_async(Global::get(), |s| Box::pin(async {
        let left = buffer.read(s, ..2, None)?.join_async()?;
        let right = buffer.read(s, 2.., None)?.join_async()?;
        println!("{left:?}");
        let v = futures::try_join!(left, right);
        println!("Done!");
        return v;
    }));


    let mut event = Box::pin(event);
    let mut ctx = core::task::Context::from_waker(futures::task::noop_waker_ref());
    let poll = blaze_rs::futures::FutureExt::poll_unpin(&mut event, &mut ctx);
    if poll.is_ready() { panic!("Ended too early") }

    drop(event);
    let mut slice = buffer.slice_mut(..)?;
    println!("Got my slice: {slice:?}");

    Ok(())
}