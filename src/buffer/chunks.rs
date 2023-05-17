use super::{Buf, BufMut, Buffer};
use crate::{
    buffer::BufferRange,
    context::{Context, Global},
    core::Result,
};

macro_rules! tri {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct ChunksExact<'a, T, C: Context = Global> {
    pub(super) buffer: &'a Buffer<T, C>,
    pub(super) offset: usize,
    pub(super) len: usize,
}

impl<'a, T, C: Context> ChunksExact<'a, T, C> {
    /// Returns the final values that don't fit inside the specified slice size.
    ///
    /// Returns `None` if there is no remainder.
    pub fn remainder(&self) -> Result<Option<Buf<'a, T, C>>>
    where
        C: Clone,
    {
        let buf_len = self.buffer.len()?;
        let rem_len = (buf_len - self.offset) % self.len;

        if rem_len == 0 {
            return Ok(None);
        }

        let range = BufferRange::from_parts::<T>(buf_len - rem_len, rem_len)?;
        return self.buffer.slice(range).map(Some);
    }
}

impl<'a, T, C: Context + Clone> Iterator for ChunksExact<'a, T, C> {
    type Item = Result<Buf<'a, T, C>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next_offset = self.offset + self.len;
        if next_offset > tri!(self.buffer.len()) {
            return None;
        }

        let range = tri!(BufferRange::from_parts::<T>(self.offset, self.len));
        let slice = tri!(self.buffer.slice(range));

        self.offset = next_offset;
        return Some(Ok(slice));
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let next_offset = self.offset + (n + 1) * self.len;
        if next_offset > tri!(self.buffer.len()) {
            return None;
        }

        let range = tri!(BufferRange::from_parts::<T>(
            self.offset + n * self.len,
            self.len
        ));
        let slice = tri!(self.buffer.slice(range));

        self.offset = next_offset;
        return Some(Ok(slice));
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<'a, T, C: Context + Clone> ExactSizeIterator for ChunksExact<'a, T, C> {
    #[inline]
    fn len(&self) -> usize {
        (self.buffer.len().expect("error finding buffer size") - self.offset) / self.len
    }
}

#[derive(Debug)]
pub struct ChunksExactMut<'a, T, C: Context = Global> {
    pub(super) buffer: &'a mut Buffer<T, C>,
    pub(super) offset: usize,
    pub(super) len: usize,
}

impl<'a, T, C: Context> ChunksExactMut<'a, T, C> {
    /// Returns the final values that don't fit inside the specified slice size.
    ///
    /// Returns `None` if there is no remainder.
    pub fn into_remainder(self) -> Result<Option<BufMut<'a, T, C>>>
    where
        C: Clone,
    {
        let buf_len = self.buffer.len()?;
        let rem_len = (buf_len - self.offset) % self.len;

        if rem_len == 0 {
            return Ok(None);
        }

        let range = BufferRange::from_parts::<T>(buf_len - rem_len, rem_len)?;
        return self.buffer.slice_mut(range).map(Some);
    }
}

impl<'a, T, C: Context + Clone> Iterator for ChunksExactMut<'a, T, C> {
    type Item = Result<BufMut<'a, T, C>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next_offset = self.offset + self.len;
        if next_offset > tri!(self.buffer.len()) {
            return None;
        }

        let range = tri!(BufferRange::from_parts::<T>(self.offset, self.len));
        let slice = unsafe {
            tri!(BufMut::from_raw(
                &self.buffer,
                range,
                self.buffer.context().clone()
            ))
        };

        self.offset = next_offset;
        return Some(Ok(slice));
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let next_offset = self.offset + (n + 1) * self.len;
        if next_offset > tri!(self.buffer.len()) {
            return None;
        }

        let range = tri!(BufferRange::from_parts::<T>(
            self.offset + n * self.len,
            self.len
        ));
        let slice = unsafe {
            tri!(BufMut::from_raw(
                &self.buffer,
                range,
                self.buffer.context().clone()
            ))
        };

        self.offset = next_offset;
        return Some(Ok(slice));
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<'a, T, C: Context + Clone> ExactSizeIterator for ChunksExactMut<'a, T, C> {
    #[inline]
    fn len(&self) -> usize {
        (self.buffer.len().expect("error finding buffer size") - self.offset) / self.len
    }
}
