use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let queue = CONTEXT.next_queue();
    let default = queue.queue_properties()?;
    
    println!("{default:?}");
    Ok(())
}