use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::{event::{listener::{SUBMITTING, RUNNING, COMPLETED, ListenerQueue}, EventStatus}, prelude::RawEvent};

static THREAD_STARTED : Flag = Flag::new(FALSE);

#[inline(always)]
pub(super) fn init_thread () {
    if THREAD_STARTED.compare_exchange(FALSE, TRUE, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_ok() {
        std::thread::spawn(thread_loop);
    }
}

fn thread_loop () {
    fn check_status (status: EventStatus, events: &Mutex<ListenerQueue>) {
        let mut events = match events.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner()
        };

        for (event, list) in events.drain_filter(|x, _| x.status().map_or(true, |x| x <= status)) {
            let status = event.status().map(|_| status);

            for f in list {
                f.call(&event, status)
            }
        }
    }

    let submitting = Lazy::force(&SUBMITTING);
    let running = Lazy::force(&RUNNING);
    let completed = Lazy::force(&COMPLETED);

    /*loop {
        check_submitting(submitting);
        check_running(running);
        check_completed(completed);
    }*/
}