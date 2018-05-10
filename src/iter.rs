use std::collections::btree_set;
use std::collections::btree_set::Iter as BIter;
use std::iter::Cloned;

use Range;

#[derive(Clone)]
pub struct IndexIter<'a> {
    free_ranges: Cloned<BIter<'a, Range>>,
    next_range: Option<Range>,
    index: usize,
    end: usize,
}

impl<'a> IndexIter<'a> {
    pub(crate) fn new(free_ranges: BIter<'a, Range>, end: usize) -> IndexIter<'a> {
        let mut free_ranges = free_ranges.cloned();
        let mut first_range = free_ranges.next();

        let mut index = 0;
        if let Some(fr) = first_range {
            if fr.min == 0 {
                index = fr.max + 1;
                first_range = free_ranges.next();
            }
        }

        IndexIter {
            free_ranges,
            next_range: first_range,
            index,
            end,
        }
    }
}

impl<'a> Iterator for IndexIter<'a> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<usize> {
        if self.index == self.end {
            return None;
        }

        let value = self.index;
        self.index += 1;

        if let Some(range) = self.next_range {
            if self.index == range.min {
                self.index = range.max + 1;
                self.next_range = self.free_ranges.next();
            }
        }

        Some(value)
    }
}

#[derive(Clone)]
pub struct IndexAfterIter<'a> {
    free_ranges: Cloned<btree_set::Range<'a, Range>>,
    next_range: Option<Range>,
    index: usize,
    end: usize,
}

impl<'a> IndexAfterIter<'a> {
    pub(crate) fn new(free_ranges: btree_set::Range<'a, Range>, start: usize, end: usize) -> IndexAfterIter<'a> {
        let mut free_ranges = free_ranges.cloned();
        let mut first_range = free_ranges.next();

        let mut index = start;
        if let Some(fr) = first_range {
            if fr.min <= start {
                index = fr.max + 1;
                first_range = free_ranges.next();
            }
        }

        IndexAfterIter {
            free_ranges,
            next_range: first_range,
            index,
            end,
        }
    }
}

impl<'a> Iterator for IndexAfterIter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        if self.index == self.end {
            return None;
        }

        let value = self.index;
        self.index += 1;

        if let Some(range) = self.next_range {
            if self.index == range.min {
                self.index = range.max + 1;
                self.next_range = self.free_ranges.next();
            }
        }

        Some(value)
    }
}
