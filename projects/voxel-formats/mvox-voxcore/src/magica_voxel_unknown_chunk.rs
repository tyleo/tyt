#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// A chunk the mvox crate does not model, preserved verbatim in the
/// `magica-voxel` ext so an unrecognized or future chunk survives the round
/// trip.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct MagicaVoxelUnknownChunk {
    /// The four-byte chunk id, as stored.
    pub id: [u8; 4],

    /// The chunk's content bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub content: Vec<u8>,

    /// The chunk's child bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub children: Vec<u8>,
}
