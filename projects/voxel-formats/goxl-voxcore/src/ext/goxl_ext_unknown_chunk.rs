#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// A chunk the goxl crate does not model, preserved verbatim in the `goxl` ext
/// so an unrecognized or future chunk survives the round trip.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxlExtUnknownChunk {
    /// The four-byte chunk type, as stored.
    pub id: [u8; 4],

    /// The chunk's data bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub data: Vec<u8>,
}
