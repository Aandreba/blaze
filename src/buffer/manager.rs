use crate::event::{WaitList, RawEvent};

#[derive(Debug, Clone)]
pub enum AccessManager {
    None,
    Reading (Vec<RawEvent>),
    Writing (RawEvent)
}

impl AccessManager {
    #[inline]
    pub fn extend_to_read (&self, wait: &mut WaitList) {
        match self {
            Self::Writing(x) => wait.extend_one(x.clone()),
            _ => {},
        }
    }

    #[inline]
    pub fn extend_to_write (&self, wait: &mut WaitList) {
        match self {
            Self::Reading(x) => wait.extend(x.iter().cloned()),
            Self::Writing(x) => wait.extend_one(x.clone()),
            _ => {},
        }
    }

    #[inline]
    pub fn read (&mut self, evt: RawEvent) {
        match self {
            Self::Reading(x) => x.push(evt),
            _ => *self = Self::Reading(vec![evt])
        }
    }

    #[inline]
    pub fn write (&mut self, evt: RawEvent) {
        *self = Self::Writing(evt)
    }
}

impl Default for AccessManager {
    #[inline(always)]
    fn default() -> Self {
        AccessManager::None
    }
}