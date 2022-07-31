use std::ops::Deref;
use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    let map = buffer.map(.., EMPTY)?.wait()?;
    
    assert_eq!(map.deref(), &[1, 2, 3, 4, 5]);
    Ok(())
}