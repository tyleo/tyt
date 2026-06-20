use crate::VXExtentRange;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-snapshot extent stat (`st.extent`). Voxel Max stores this as a
/// dictionary keyed by an order tag `{o: <order>}` — a constant `5` for the 32³
/// chunk grid (2⁵ = 32) — not as a vector, so it is modeled as a struct to match
/// that shape exactly (a 3-element array here makes Voxel Max's decoder crash).
/// Voxel Max-authored snapshots also carry the occupied range `r`.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXExtent {
    /// Order tag (`o`); observed constant `5` (2⁵ = 32, the chunk edge length).
    pub o: i64,
    /// Occupied-voxel range (`r`); present on Voxel Max-authored snapshots, absent
    /// from the minimal `{o}` extent that `from-voxj` writes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub r: Option<VXExtentRange>,
}
