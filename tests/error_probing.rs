use blaze_rs::{prelude::*, context::CommandQueue};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () -> Result<()> {
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    let queue = CommandQueue::new(CONTEXT.next_queue().clone());
    let flag = FlagEvent::new()?;

    println!("{}", queue.size());
    queue.enqueue(|_, b| buffer.read(.., b), &flag);

    Ok(())
}