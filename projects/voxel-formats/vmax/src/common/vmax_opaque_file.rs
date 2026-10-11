#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The stored bytes of a package file this crate keeps without reading, such
/// as `animations.vmaxa` or an external mesh. Voxel Max keeps a file it does
/// not know untouched, so the bytes round-trip verbatim.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxOpaqueFile(pub Vec<u8>);
