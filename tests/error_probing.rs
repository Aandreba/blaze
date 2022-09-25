#![feature(nonzero_min_max)]
use blaze_proc::join_various_blocking;
use blaze_rs::{prelude::*};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    use std::ops::Deref;
    
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    
    let (left, right) = scope(|s| {
        let left = buffer.read(s, 2.., None)?;
        let right = buffer.map(s, 2.., None)?;
        return join_various_blocking!(left, right)
    })?;

    assert_eq!(left.as_slice(), right.deref());
    Ok(())
}

#[cfg(feature = "cl1_2")]
#[test]
fn test () -> Result<()> {
    let mut buffer = blaze_rs::buffer![|i| i; 10]?;

    let v = scope(|s| {
        let (evt, abort) = buffer.read(s, 1..=4, None)?.abortable()?;
        return Ok(abort);
    })?;
    
    //println!("{left:?}, {right:?}");
    Ok(())
}

// TODO Rethink system for non-blocking events
#[cfg(feature = "futures")]
#[tokio::test]
async fn async_test () -> Result<()> {
    use blaze_rs::{buffer, scope_async};
    use futures::{task::*, future::*};
    
    let buffer = buffer![1, 2, 3, 4, 5]?;
    
    let mut scope = Box::pin(scope_async!(|s| async {
        let left = buffer.read(s, ..2, None)?.inspect(|_| println!("Left done!")).join_async()?.await;
        let right = buffer.read(s, ..2, None)?.inspect(|_| println!("Right done!")).join_async()?.await;
        return Ok((left, right));
    }));
    
    let mut ctx = std::task::Context::from_waker(noop_waker_ref());
    let _ = scope.poll_unpin(&mut ctx)?;
    drop(scope); // prints "Left done!", doesn't print "Right done!"
    Ok(())
}

#[test]
fn rect () -> Result<()> {
    let buff = RectBuffer2D::new(&[1, 2, 3, 4, 5, 6], 2, MemAccess::READ_WRITE, false)?;
    println!("{buff:?}");
    Ok(())
}