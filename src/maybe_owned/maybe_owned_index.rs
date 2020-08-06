use super::MaybeOwned;
use std::ops::{Index, Range};

type IndexWidth = u16;

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MaybeOwnedIndex {
    pub start: IndexWidth,
    pub end: IndexWidth,
}

// TODO document this
impl MaybeOwnedIndex {
    pub const fn raw(start: usize, end: usize) -> Self {
        Self {
            start: start as IndexWidth,
            end: end as IndexWidth,
        }
    }

    pub const fn new(pos: usize) -> Self {
        Self {
            start: pos as IndexWidth,
            end: pos as IndexWidth,
        }
    }

    pub const fn offset_by(mut self, pos: usize) -> Self {
        self.start += pos as IndexWidth;
        self.end += pos as IndexWidth;
        self
    }

    pub const fn resize(mut self, len: usize) -> Self {
        self.end = self.start + len as IndexWidth;
        self
    }

    pub const fn truncate(mut self, len: usize) -> Self {
        self.end -= len as IndexWidth;
        self
    }

    pub fn replace(&mut self, pos: usize) -> MaybeOwnedIndex {
        std::mem::replace(self, Self::new(pos))
    }

    pub const fn is_empty(&self) -> bool {
        // end can never be behind start
        // so if we're past start then we're not empty
        self.start == self.end
    }

    pub fn bump_tail(&mut self) {
        self.end += 1;
    }

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
    type Output = str;
    fn index(&self, index: &MaybeOwnedIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}

impl<'a> Index<MaybeOwnedIndex> for str {
    type Output = str;
    fn index(&self, index: MaybeOwnedIndex) -> &Self::Output {
        &self[index.as_range()]
    }
}
