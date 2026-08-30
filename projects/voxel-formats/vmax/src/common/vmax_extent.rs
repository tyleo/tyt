use crate::VMaxExtentRange;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-snapshot extent stat (`st.extent`).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxExtent {
    /// Order tag; observed constant `5` (2^5 = 32, the chunk edge length).
    pub o: i64,

    /// Occupied-voxel range.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub r: Option<VMaxExtentRange>,
}
