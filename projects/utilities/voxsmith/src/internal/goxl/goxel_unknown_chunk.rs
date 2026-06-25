use serde::{Deserialize, Serialize};

/// A chunk the goxl crate does not model, preserved verbatim in the `goxel` ext
/// so an unrecognized or future chunk survives the round trip.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GoxelUnknownChunk {
    /// The four-byte chunk type, as stored.
    pub id: [u8; 4],

    /// The chunk's data bytes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<u8>,
}
