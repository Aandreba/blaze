flat_mod!(raw, flags, global, single, queue);

use std::{ops::Deref, marker::PhantomData, sync::{atomic::{AtomicUsize, AtomicBool}, Arc}};
use crate::prelude::{Result, RawCommandQueue, RawEvent, Event};

/// An object that can be used as a Blaze context, with a similar syntax to Rust allocators.\
/// Blaze contexts are similar to OpenCL contexts, except they're also in charge of administrating and supplying
/// their various command queues. This allows Blaze contexts to manage the load between the various devices in an
/// OpenCL context. 
pub trait Context: Deref<Target = RawContext> {
    fn queues (&self) -> &[CommandQueue];
    fn next_queue (&self) -> &CommandQueue;

    #[inline(always)]
    fn as_raw (&self) -> &RawContext {
        self
    }

    #[inline(always)]
    fn flush_all (&self) -> Result<()> {
        for queue in self.queues() {
            queue.flush()?
        }

        Ok(())
    }

    #[inline(always)]
    fn finish_all (&self) -> Result<()> {
        for queue in self.queues() {
            queue.finish()?
        }

        Ok(())
    }
}

#[inline(always)]
pub fn scope<'env, F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>)> (f: F) {
    local_scope(Global::get(), f)
}

pub fn local_scope<'ctx, 'env, C: 'ctx + Context, F: for<'scope> FnOnce(&'scope LocalScope<'ctx, 'scope, 'env, C>)> (c: &'ctx C, f: F) {
    // todo panic detection
    let scope = LocalScope {
        ctx,
        size: Size
    };

    todo!()
}

pub type Scope<'scope, 'env> = LocalScope<'static, 'scope, 'env, Global>;

pub struct LocalScope<'ctx, 'scope, 'env: 'scope, C: 'ctx + Context = Global> {
    ctx: &'ctx C,
    size: Size,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>
}

impl<'ctx, 'scope, 'env: 'scope, C: 'ctx + Context> LocalScope<'ctx, 'scope, 'env, C> {
    pub fn enqueue<T, E: FnOnce(&'ctx RawCommandQueue) -> Result<RawEvent>, F: 'scope + FnOnce() -> Result<T>> (&'scope self, supplier: E, f: F) -> Result<Event<'scope, T>> {
        let queue = self.ctx.next_queue();
        let inner = supplier(&queue)?;
        let evt = Event::new(inner, f);

        let queue_size = queue.size.clone();
        let scope_size = self.size.clone();
        evt.on_complete(move |_, _| {
            drop(queue_size);
            drop(scope_size);
        }).unwrap();

        return Ok(evt)
    }
}

/*
```rust
#[global_context]
static CTX : SimpleContext = SimpleContext::default();

fn manually () {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false);

    scope(|s| {
        let _ = buffer.write(s, 2, &[6], &[]);
    });

    scope(|s| {
        let v = buffer.read(s, .., &[])?.join()?;
        assert_eq!(v.as_slice(), &[1, 2, 6, 4, 5]);
    })
}

#[scoped(s)] // it may default to `s`
fn auto_v1 () {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false);
    
    scope(|s| {
        let _ = buffer.write(s, 2, &[6], &[]);
        todo!();
    });

    let v = buffer.read(s, .., &[])?.join()?;
    assert_eq!(v.as_slice(), &[1, 2, 6, 4, 5]);
}
```
*/