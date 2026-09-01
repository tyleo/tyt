use crate::{Error, Result};

/// An inclusive range of indices, one selector over a document's indexed
/// entries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexRange {
    start: usize,
    end: usize,
}

impl IndexRange {
    /// The range `start..=end`. Errors when `start` exceeds `end`.
    pub fn new(start: usize, end: usize) -> Result<Self> {
        if start > end {
            return Err(Error::invalid(format!(
                "range start {start} is greater than its end {end}"
            )));
        }

        Ok(IndexRange { start, end })
    }

    /// Whether `index` falls in the range.
    pub fn contains(self, index: usize) -> bool {
        self.start <= index && index <= self.end
    }
}
