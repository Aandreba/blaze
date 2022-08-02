use blaze_rs::{prelude::*, context::CommandQueue};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () -> Result<()> {
    let queue = CommandQueue::new(CONTEXT.next_queue().clone());
    let flag = FlagEvent::new()?;

    println!("{}", queue.size());

    Ok(())
}