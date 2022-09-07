#![feature(nonzero_min_max)]

use blaze_rs::{prelude::*};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    buffer.write_blocking(1, &[9, 8], &[])?;
    println!("{buffer:?}");

    let [left, right] = scope(|s| {
        let left = buffer.read(s, ..2, &[])?;
        let right = buffer.read(s, 2.., &[])?;
        return Event::join_all_sized_blocking([left, right])
    })?;

    println!("{left:?} {right:?}");

    // Problem. The status of read is unknown
    //let write = buffer.write(2, &[2], &[])?;

    Ok(())
}