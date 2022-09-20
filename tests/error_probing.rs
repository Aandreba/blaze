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
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    
    let [left, right] = scope(|s| {
        let left = buffer.read(s, ..2, None)?;
        let right = buffer.read(s, 2.., None)?;
        Event::join_all_sized_blocking([left, right])
    })?;
    
    println!("{left:?}, {right:?}");
    Ok(())
}

#[cfg(feature = "futures")]
#[tokio::test]
async fn async_test () -> Result<()> {
    use blaze_rs::futures::FutureExt;

    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    let mut event = Box::pin(blaze_rs::scope_async!(|s| async {
        let left = buffer.read(s, ..2, None)?.join_async()?;
        let right = buffer.read(s, 2.., None)?.join_async()?;
        futures::try_join!(left, right)
    }));

    let mut ctx = core::task::Context::from_waker(futures::task::noop_waker_ref());

    loop {
        let poll = event.poll_unpin(&mut ctx);
        println!("{poll:?}");
        if poll.is_ready() { break }
    }
    
    Ok(())
}