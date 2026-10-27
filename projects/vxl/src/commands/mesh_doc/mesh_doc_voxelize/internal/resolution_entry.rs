use crate::NamedCliValue;
use serde::Deserialize;
use std::num::NonZeroU32;
use voxsmith::utilities::{GridResolution, ResolutionReference};

/// A profile's `--resolution`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResolutionEntry {
    pub(crate) reference: NamedCliValue<ResolutionReference>,

    pub(crate) count: NonZeroU32,
}

impl From<ResolutionEntry> for GridResolution {
    fn from(entry: ResolutionEntry) -> Self {
        GridResolution::ReferenceCount {
            reference: entry.reference.0,
            count: entry.count.get(),
        }
    }
}
