//! A pool which manages allocation of unique indices. Acts like a
//! psuedo-memory allocator.
//!
//! ```
//! extern crate index_pool;
//! use index_pool::IndexPool;
//! 
//! fn main() {
//!     let mut pool = IndexPool::new();
//! 
//!     let a = pool.new_id();
//!     let b = pool.new_id();
//!     let c = pool.new_id();
//! 
//!     let mut data = vec![""; pool.maximum()];
//!     data[a] = "apple";
//!     data[b] = "banana";
//!     data[c] = "coconut";
//! 
//!     // Nevermind, no bananas
//!     pool.return_id(b).unwrap();
//! 
//!     let p = pool.new_id();
//!     data[p] = "pineapple";
//! 
//!     assert_eq!(data, vec!["apple", "pineapple", "coconut"]);
//! }
//! ```

use std::collections::BTreeSet;
use std::cmp::{self, Ordering};
use std::error::Error;
use std::fmt;

/// A pool which manages allocation of unique indices. Acts like a
/// psuedo-memory allocator.
#[derive(Debug)]
pub struct IndexPool {
    next_id: usize,
    free_list: BTreeSet<Range>,
}

impl IndexPool {
    /// Constructs an empty IndexPool. Indices will start at `0`.
    #[inline]
    pub fn new() -> Self {
        Self::with_initial_index(0)
    }

    /// Constructs an empty IndexPool. `index` will be the first
    /// index returned from `new_id`. You can logically think of
    /// this as either specifying a base index for the pool, or
    /// pre-allocating the `[0..index)` range. This datastructure
    /// does not care which is your usecase, and neither has any
    /// kind of performance penalty.
    pub fn with_initial_index(index: usize) -> Self {
        IndexPool {
            next_id: index,
            free_list: BTreeSet::new(),
        }
    }

    /// Allocates a new index for use. This is guaranteed to not be any index
    /// which has previously been returned from `new_id` but has not yet been
    /// passed to `return_id`.
    #[inline]
    pub fn new_id(&mut self) -> usize {
        if let Some(first_range) = self.free_list.iter().nth(0).cloned() {
            self.free_list.remove(&first_range);
            let reduced = first_range.pop_front();
            if !reduced.empty() {
                self.free_list.insert(reduced);
            }
            return first_range.min;
        }

        let id = self.next_id;
        self.next_id += 1;
        return id;
    }

    /// Gives an Id back to the pool so that it may be handed out again.
    /// Returns Err if the Id was not in use at the time. Whether ignoring
    /// such an error is okay is up to your own usecase.
    #[inline]
    pub fn return_id(&mut self, id: usize) -> Result<(), AlreadyReturned> {
        if id >= self.next_id || self.free_list.contains(&Range::id(id)) {
            return Err(AlreadyReturned);
        }

        if id + 1 == self.next_id {
            self.next_id -= 1;
        } else {
            self.set_free(id);
        }

        while self.collapse_next() {}

        Ok(())
    }

    /// Returns an upper bound on the number of IDs which have been
    /// allocated, specifically the `highest numbered ID in use + 1`.
    /// Useful if you're going to e.g. create a Vec which has room
    /// for all of your IDs.
    #[inline]
    pub fn maximum(&self) -> usize {
        self.next_id
    }

    #[inline]
    fn set_free(&mut self, id: usize) {
        let range = Range::id(id);

        let range_front = if id > 0 { range.push_front() } else { range };
        let range_back = range.push_back();
        let combine_front = self.free_list.get(&range_front).cloned();
        let combine_back = self.free_list.get(&range_back).cloned();

        match (combine_front, combine_back) {
            (Some(front_range), Some(back_range)) => {
                let combined = front_range.merge(range).merge(back_range);

                self.free_list.remove(&front_range);
                self.free_list.remove(&back_range);
                self.free_list.insert(combined);
            }
            (Some(front_range), None) => {
                let combined = front_range.merge(range);

                self.free_list.remove(&front_range);
                self.free_list.insert(combined);
            }
            (None, Some(back_range)) => {
                let combined = back_range.merge(range);

                self.free_list.remove(&back_range);
                self.free_list.insert(combined);
            }
            (None, None) => {
                self.free_list.insert(range);
            }
        }
    }

    #[inline]
    fn collapse_next(&mut self) -> bool {
        if let Some(last_range) = self.free_list.iter().rev().nth(0).cloned() {
            if last_range.max + 1 == self.next_id {
                self.free_list.remove(&last_range);
                self.next_id = last_range.min;
                return true;
            }
        }

        false
    }
}

impl Default for IndexPool {
    /// Constructs an empty IndexPool. Indices will start at `0`.
    #[inline]
    fn default() -> Self {
        IndexPool::new()
    }
}

#[derive(Debug)]
pub struct AlreadyReturned;

impl fmt::Display for AlreadyReturned {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for AlreadyReturned {
    fn description(&self) -> &str {
        "An index was tried to be returned to the pool, but it was already marked as free."
    }
}

#[derive(Copy, Clone)]
struct Range {
    min: usize,
    max: usize,
}

impl fmt::Debug for Range {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}...{})", self.min, self.max)
    }
}

impl Range {
    #[inline]
    fn id(id: usize) -> Self {
        Range { min: id, max: id }
    }

    #[inline]
    fn empty(self) -> bool {
        self.min > self.max
    }

    #[inline]
    fn push_front(mut self) -> Self {
        self.min -= 1;
        self
    }

    #[inline]
    fn push_back(mut self) -> Self {
        self.max += 1;
        self
    }

    #[inline]
    fn pop_front(mut self) -> Self {
        self.min += 1;
        self
    }

    #[inline]
    fn merge(self, other: Self) -> Self {
        Range {
            min: cmp::min(self.min, other.min),
            max: cmp::max(self.max, other.max),
        }
    }

    #[inline]
    fn contains(&self, value: usize) -> bool {
        value >= self.min && value <= self.max
    }
}

impl PartialEq for Range {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Range {}

impl PartialOrd for Range {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Range {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.contains(other.min) || self.contains(other.max) {
            return Ordering::Equal;
        }

        self.min.cmp(&other.min)
    }
}
