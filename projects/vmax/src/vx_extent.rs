use crate::VXExtentRange;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The per-snapshot extent stat (`st.extent`). Voxel Max stores this as a
/// dictionary keyed by an order tag `{o: <order>}`, a constant `5` for the 32^3
/// chunk grid (2^5 = 32).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXExtent {
    /// Order tag; observed constant `5` (2^5 = 32, the chunk edge length).
    pub o: i64,

    /// Occupied-voxel range.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub r: Option<VXExtentRange>,
}
