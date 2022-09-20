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
    let slice = buffer.slice(1..)?;
    let map = slice.map_blocking(1.., None)?;
    
    println!("{slice:?}");
    println!("{map:?}");
    Ok(())
}

#[cfg(feature = "futures")]
#[tokio::test]
async fn async_test () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    

    todo!()
}