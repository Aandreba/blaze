use std::{sync::{Arc, atomic::{AtomicUsize, Ordering, AtomicI32}}, marker::PhantomData, panic::{catch_unwind, AssertUnwindSafe, resume_unwind}, thread::Thread};
use opencl_sys::CL_SUCCESS;

use crate::{prelude::{Result, RawCommandQueue, RawEvent, Event, Error}, event::{Consumer, NoopEvent, Noop}};
use super::{Global, Context};

pub struct Scope<'scope, 'env: 'scope, C: 'env + Context = Global> {
    ctx: &'env C,
    data: Arc<(AtomicUsize, AtomicI32)>,
    thread: Thread,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>
}

impl<'scope, 'env: 'scope, C: 'env + Context> Scope<'scope, 'env, C> {
    pub fn enqueue<T, E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>, F: Consumer<'scope, T>> (&'scope self, supplier: E, consumer: F) -> Result<Event<T, F>> {
        let queue = self.ctx.next_queue();
        let inner = supplier(&queue)?;
        let evt = Event::new(inner, consumer);

        if self.data.0.fetch_add(1, Ordering::Relaxed) == usize::MAX {
            panic!("too many events in scope")
        }

        let queue_size = queue.size.clone();
        let scope_data = self.data.clone();
        let scope_thread = self.thread.clone();

        evt.on_complete(move |_, res| {
            drop(queue_size);

            if let Err(e) = res {
                let _ = scope_data.1.compare_exchange(CL_SUCCESS, e.ty as i32, Ordering::Relaxed, Ordering::Relaxed);
            }

            if scope_data.0.fetch_sub(1, Ordering::Relaxed) == 1 {
                scope_thread.unpark();
            }
        }).unwrap();

        return Ok(evt)
    }

    #[inline(always)]
    pub fn enqueue_noop<E: FnOnce(&'env RawCommandQueue) -> Result<RawEvent>> (&'scope self, supplier: E) -> Result<NoopEvent<'scope>> {
        self.enqueue(supplier, Noop::new())
    }
}

#[inline(always)]
pub fn scope<'env, T, F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> Result<T>> (f: F) -> Result<T> {
    local_scope(Global::get(), f)
}

pub fn local_scope<'env, T, C: 'env + Context, F: for<'scope> FnOnce(&'scope Scope<'scope, 'env, C>) -> Result<T>> (ctx: &'env C, f: F) -> Result<T> {
    let mut scope = Scope {
        ctx,
        data: Arc::new((AtomicUsize::new(0), AtomicI32::new(CL_SUCCESS))),
        thread: std::thread::current(),
        scope: PhantomData,
        env: PhantomData
    };

    // Run `f`, but catch panics so we can make sure to wait for all the threads to join.
    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));
    
    // Wait until all the events are finished.
    while scope.data.0.load(Ordering::Acquire) != 0 {
        std::thread::park();
    }

    // Throw any panic from `f`, or the return value of `f`.
    return match result {
        Err(e) => resume_unwind(e),
        Ok(x) => {
            let e = scope.data.1.load(Ordering::Relaxed);
            if e != CL_SUCCESS {
                return Err(Error::from(e));
            }
            x
        }
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