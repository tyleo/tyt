use crate::VMaxStorage;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single entry in a
/// [`VMaxContentsVmaxbFile`](crate::VMaxContentsVmaxbFile)'s
/// [`snapshots`](crate::VMaxContentsVmaxbFile::snapshots) array.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxSnapshot {
    pub s: VMaxStorage,
}
