#![feature(nonzero_min_max)]

use blaze_rs::prelude::*;
use rand::random;
use tokio::spawn;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    
    let evt = buffer.read(.., &[])?;
    let read = evt.join();
    println!("{read:?}");
    // Problem. The status of read is unknown
    //let write = buffer.write(2, &[2], &[])?;

    Ok(())
}