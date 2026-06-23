use crate::VMaxSerdeStorage;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single entry in a
/// [`VMaxSerdeContentsVmaxbFile`](crate::VMaxSerdeContentsVmaxbFile)'s
/// [`snapshots`](crate::VMaxSerdeContentsVmaxbFile::snapshots) array.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxSerdeSnapshot {
    pub s: VMaxSerdeStorage,
}
