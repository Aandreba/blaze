use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, marker::PhantomData, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}, thread::Thread};
use crate::{prelude::{Result, RawCommandQueue, RawEvent, Event}, event::{Consumer, NoopEvent, Noop}};
use super::{Global, Context};

pub type Scope<'scope, 'env> = LocalScope<'static, 'scope, 'env, Global>;

pub struct LocalScope<'ctx: 'scope, 'scope, 'env: 'scope, C: 'ctx + Context> {
    ctx: &'ctx C,
    size: Arc<AtomicUsize>,
    thread: Thread,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>
}

impl<'ctx, 'scope, 'env: 'scope, C: 'ctx + Context> LocalScope<'ctx, 'scope, 'env, C> {
    pub fn enqueue<T, E: FnOnce(&'ctx RawCommandQueue) -> Result<RawEvent>, F: Consumer<'scope, T>> (&'scope self, supplier: E, consumer: F) -> Result<Event<T, F>> {
        let queue = self.ctx.next_queue();
        let inner = supplier(&queue)?;
        let evt = Event::new(inner, consumer);

        let queue_size = queue.size.clone();
        let scope_size = self.size.clone();
        let scope_thread = self.thread.clone();

        evt.on_complete(move |_, _| {
            drop(queue_size);
            if scope_size.fetch_sub(1, Ordering::AcqRel) == 1 {
                scope_thread.unpark();
            }
        }).unwrap();

        return Ok(evt)
    }

    #[inline(always)]
    pub fn enqueue_noop<E: FnOnce(&'ctx RawCommandQueue) -> Result<RawEvent>> (&'scope self, supplier: E) -> Result<NoopEvent<'scope>> {
        self.enqueue(supplier, Noop::new())
    }
}

#[inline(always)]
pub fn scope<'env, T, F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> Result<T>> (f: F) -> Result<T> {
    local_scope(Global::get(), f)
}

pub fn local_scope<'ctx, 'env, T, C: 'ctx + Context, F: for<'scope> FnOnce(&'scope LocalScope<'ctx, 'scope, 'env, C>) -> Result<T>> (ctx: &'ctx C, f: F) -> Result<T> {
    let scope = LocalScope {
        ctx,
        size: Arc::new(AtomicUsize::new(0)),
        thread: std::thread::current(),
        scope: PhantomData,
        env: PhantomData
    };

    // Run `f`, but catch panics so we can make sure to wait for all the threads to join.
    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));
    
    // Wait until all the events are finished.
    while scope.size.load(Ordering::Acquire) != 0 {
        std::thread::park();
    }

    // Throw any panic from `f`, or the return value of `f`.
    return match result {
        Err(e) => resume_unwind(e),
        Ok(x) => x
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