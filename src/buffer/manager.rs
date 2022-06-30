use crate::event::{WaitList, RawEvent};

#[derive(Clone)]
pub enum AccessManager {
    None,
    Reading (Vec<RawEvent>),
    Writing (RawEvent)
}

impl AccessManager {
    #[inline]
    pub fn extend_list (&self, wait: &mut WaitList) {
        match self {
            Self::Reading(x) => wait.extend(x.into_iter().cloned()),
            Self::Writing(x) => wait.extend_one(x.clone()),
            Self::None => {},
        }
    }

    #[inline]
    pub fn read (&mut self, evt: RawEvent) -> WaitList {
        match self {
            Self::None => {
                *self = Self::Reading(vec![evt]);
                WaitList::EMPTY
            },

            Self::Reading(x) => {
                x.push(evt);
                WaitList::EMPTY
            },

            Self::Writing(x) => {
                let wait = WaitList::from_event(x.clone());
                *self = Self::Reading(vec![evt]);
                wait
            }
        }
    }

    #[inline]
    pub fn write (&mut self, evt: RawEvent) -> WaitList {
        match self {
            Self::None => {
                *self = Self::Writing(evt);
                WaitList::EMPTY
            },

            Self::Reading(x) => {
                let wait = WaitList::new(core::mem::take(x));
                *self = Self::Writing(evt);
                wait
            },

            Self::Writing(x) => WaitList::from_event(core::mem::replace(x, evt))
        }
    }
}