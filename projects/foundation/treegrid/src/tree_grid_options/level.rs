use crate::TreeGridOptions;
use std::num::NonZeroU8;

impl TreeGridOptions {
    pub(crate) fn level(&self) -> NonZeroU8 {
        self.header_level.unwrap_or(NonZeroU8::MIN)
    }
}
