use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::{event::{listener::{SUBMITTING, RUNNING, COMPLETED, ListenerQueue}, EventStatus}, prelude::{RawEvent, Event}};
use super::{Flag, FALSE, TRUE};

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

        let events = events.as_mut_queue();

        let mut i = 0;
        while i < events.len() {
            let status_result = events[i].0.status();

            if status_result.is_err() || status_result.is_ok_and(|x| x <= &status) {
                let status_result = status_result.map(|_| status);
                let (event, listeners) = unsafe {
                    events.remove(i).unwrap_unchecked()
                };

                for f in listeners {
                    f.call(&event, status_result.clone());
                }

                continue
            }

            i += 1;
        }
    }

    loop {
        check_status(EventStatus::Submitted, &SUBMITTING);
        check_status(EventStatus::Submitted, &RUNNING);
        check_status(EventStatus::Complete, &COMPLETED);
    }
}