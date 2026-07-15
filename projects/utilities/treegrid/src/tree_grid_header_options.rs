use std::num::NonZeroU8;

/// Options `header` labels carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeGridHeaderOptions {
    /// The level of the shallowest heading, default `1`. A heading
    /// that nests past level `6`, markdown's deepest, renders as a
    /// bold label (`**label**`) instead of a deeper `#` run.
    pub level: NonZeroU8,
}

impl TreeGridHeaderOptions {
    /// Sets the shallowest heading level.
    pub fn with_level(mut self, level: NonZeroU8) -> Self {
        self.level = level;
        self
    }
}

impl Default for TreeGridHeaderOptions {
    fn default() -> Self {
        Self {
            level: NonZeroU8::MIN,
        }
    }
}
