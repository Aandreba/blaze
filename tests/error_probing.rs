use blaze::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    
    let mut map = buffer.map_mut(.., EMPTY)?.wait()?;
    map[0] = 123;
    drop(map);

    println!("{buffer:?}");
    Ok(())
}