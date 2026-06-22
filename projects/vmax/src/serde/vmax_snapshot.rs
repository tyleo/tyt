use crate::VMaxStorage;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single entry in a
/// [`VMaxContentsVmaxbFile`](crate::VMaxContentsVmaxbFile)'s
/// [`snapshots`](crate::VMaxContentsVmaxbFile::snapshots) array.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxSnapshot {
    pub s: VMaxStorage,
}
