use std::ops::Deref;
use blaze::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    let map = buffer.map(.., EMPTY)?.wait()?;
    let mut_map = buffer.map_mut(.., EMPTY)?.wait()?; // compile error: cannot borrow `buffer` as mutable because it is also borrowed as immutable

    assert_eq!(map.deref(), &[1, 2, 3, 4, 5]);
    Ok(())
}