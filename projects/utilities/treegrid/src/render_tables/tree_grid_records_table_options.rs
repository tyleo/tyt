use std::num::NonZeroU8;

/// Options record tables consume.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeGridRecordsTableOptions {
    /// The section headings' level, default `1`. A level past `6`
    /// renders bold labels.
    pub level: NonZeroU8,
}

impl TreeGridRecordsTableOptions {
    /// Sets the section heading level.
    pub fn with_level(mut self, level: NonZeroU8) -> Self {
        self.level = level;
        self
    }
}

impl Default for TreeGridRecordsTableOptions {
    fn default() -> Self {
        Self {
            level: NonZeroU8::MIN,
        }
    }
}
