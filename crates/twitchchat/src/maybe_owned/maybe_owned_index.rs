use super::MaybeOwned;
use std::ops::{Index, Range};

type IndexWidth = u16;

/// An index into a [`MaybeOwned`].
#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MaybeOwnedIndex {
    /// The start index
    pub start: IndexWidth,
    /// The end index
    pub end: IndexWidth,
}

impl MaybeOwnedIndex {
    /// Create a new index from this start and end point
    pub const fn raw(start: usize, end: usize) -> Self {
        Self {
            start: start as IndexWidth,
            end: end as IndexWidth,
        }
    }

    /// Create a new index with the same starting/ending point.
    ///
    /// This has the end point to start -- so you can resize/bump/etc the end.
    pub const fn new(pos: usize) -> Self {
        Self {
            start: pos as IndexWidth,
            end: pos as IndexWidth,
        }
    }

    /// Shift the whole start/end pairs by `pos` amount
    pub const fn offset_by(mut self, pos: usize) -> Self {
        self.start += pos as IndexWidth;
        self.end += pos as IndexWidth;
        self
    }

    /// Grow the end by `len` amount
    pub const fn resize(mut self, len: usize) -> Self {
        self.end = self.start + len as IndexWidth;
        self
    }

    /// Shrink the end by `len` amount
    pub const fn truncate(mut self, len: usize) -> Self {
        self.end -= len as IndexWidth;
        self
    }

    /// Replace this index with a new one start/ending at `pos`, returning the old index
    pub fn replace(&mut self, pos: usize) -> Self {
        std::mem::replace(self, Self::new(pos))
    }

    /// Checks whether this index is empty (e.g. start points to the dn)
    pub const fn is_empty(&self) -> bool {
        // end can never be behind start
        // so if we're past start then we're not empty
        self.start == self.end
    }

    /// Bump the 'end' by 1 unit
    pub fn bump_tail(&mut self) {
        self.end += 1;
    }

    /// Get this type as a range
    pub const fn as_range(self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }
}

impl<'a> Index<&MaybeOwnedIndex> for MaybeOwned<'a> {
    type Output = str;
    fn index(&self, index: &MaybeOwnedIndex) -> &Self::Output {
        &self.as_ref()[index.as_range()]
    }
}

impl<'a> Index<MaybeOwnedIndex> for MaybeOwned<'a> {
    type Output = str;
    fn index(&self, index: MaybeOwnedIndex) -> &Self::Output {
        &self.as_ref()[index.as_range()]
    }
}

impl<'a> Index<&MaybeOwnedIndex> for str {
    type Output = Self;
    fn index(&self, index: &MaybeOwnedIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}

impl<'a> Index<MaybeOwnedIndex> for str {
    type Output = Self;
    fn index(&self, index: MaybeOwnedIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}
