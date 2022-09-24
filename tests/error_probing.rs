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
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;

    let event = blaze_rs::scope_async!(
        |s| async {
            let left = buffer.map(s, ..2, None)?.inspect::<fn(&_)>(|_| println!("Done 1")).join_async()?;
            let right = buffer.map(s, 2.., None)?.inspect::<fn(&_)>(|_| println!("Done 2")).join_async()?;
            println!("Wait");
            let v = futures::try_join!(left, right);
            println!("Done!");
            return v;
        }
    );

    println!("{}", core::mem::size_of_val(&event));
    let v = event.await;
    //let mut slice = buffer.slice_mut(1..)?;
    //println!("Got my slice: {slice:?}");
    println!("{v:?}");

    Ok(())
}

#[test]
fn rect () -> Result<()> {
    let buff = BufferRect2D::new(&[1, 2, 3, 4, 5, 6], 2, MemAccess::READ_WRITE, false)?;
    println!("{buff:?}");
    Ok(())
}