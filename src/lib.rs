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

extern crate free_ranges;

use free_ranges::FreeRanges;
use free_ranges::Range;

use std::error::Error;
use std::fmt;
use std::usize;

pub mod iter;

/// A pool which manages allocation of unique indices. Acts like a
/// psuedo-memory allocator.
#[derive(Debug)]
pub struct IndexPool {
    next_id: usize,
    in_use: usize,
    free_list: FreeRanges,
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
    /// kind of performance penalty, except that `in_use()` will
    /// include the `[0..index)` range.
    pub fn with_initial_index(index: usize) -> Self {
        IndexPool {
            next_id: index,
            in_use: 0,
            free_list: FreeRanges::new(),
        }
    }

    /// Allocates a new index for use. This is guaranteed to not be any index
    /// which has previously been returned from `new_id` but has not yet been
    /// passed to `return_id`.
    #[inline]
    pub fn new_id(&mut self) -> usize {
        self.in_use += 1;

        if let Some(id) = self.free_list.set_first_used() {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;
        return id;
    }

    #[inline]
    /// Attempts to allocate a specific index
    pub fn request_id(&mut self, id: usize) -> Result<(), AlreadyInUse> {
        assert!(id < usize::MAX);
        if id == self.next_id {
            self.next_id += 1;
            self.in_use += 1;
            Ok(())
        } else if id > self.next_id {
            self.free_list.set_range_free(Range {
                min: self.next_id,
                max: id - 1,
            });
            self.next_id = id + 1;
            self.in_use += 1;

            Ok(())
        } else if self.free_list.set_used(id) {
            self.in_use += 1;
            Ok(())
        } else {
            Err(AlreadyInUse)
        }
    }

    /// Gives an Id back to the pool so that it may be handed out again.
    /// Returns Err if the Id was not in use at the time. Whether ignoring
    /// such an error is okay is up to your own usecase.
    #[inline]
    pub fn return_id(&mut self, id: usize) -> Result<(), AlreadyReturned> {
        if id >= self.next_id {
            return Err(AlreadyReturned);
        }

        if id + 1 == self.next_id {
            self.next_id -= 1;
        } else {
            if !self.free_list.set_free(id) {
                return Err(AlreadyReturned);
            }
            assert!(self.free_list.is_free(id));
        }

        self.in_use -= 1;

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

    /// Returns the number of currently in-use indices
    #[inline]
    pub fn in_use(&self) -> usize {
        self.in_use
    }

    #[inline]
    /// Checks if a specific index is currently free
    pub fn is_free(&self, id: usize) -> bool {
        id >= self.next_id || self.free_list.is_free(id)
    }

    /// Returns an iterator over all indices which are in use
    #[inline]
    pub fn all_indices(&self) -> iter::IndexIter {
        iter::IndexIter::new(self.free_list.free_ranges(), self.next_id)
    }

    #[inline]
    pub fn all_indices_after(&self, after: usize) -> iter::IndexAfterIter {
        iter::IndexAfterIter::new(self.free_list.free_ranges_after(after), after, self.next_id)
    }

    #[inline]
    fn collapse_next(&mut self) -> bool {
        if let Some(last_range) = self.free_list.free_ranges().rev().nth(0).cloned() {
            if last_range.max + 1 == self.next_id {
                self.free_list.remove_last_contiguous();
                self.next_id = last_range.min;
                return true;
            }
        }

        false
    }

    #[inline]
    pub fn clear(&mut self) {
        self.free_list.clear();
        self.in_use = 0;
        self.next_id = 0;
    }
}

impl Default for IndexPool {
    /// Constructs an empty IndexPool. Indices will start at `0`.
    #[inline]
    fn default() -> Self {
        IndexPool::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyInUse;

impl fmt::Display for AlreadyInUse {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for AlreadyInUse {
    fn description(&self) -> &str {
        "An index was requested which was already marked as in use."
    }
}
