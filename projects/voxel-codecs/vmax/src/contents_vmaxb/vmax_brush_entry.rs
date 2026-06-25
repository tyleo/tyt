use crate::VMaxBrushColor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One slot in a Voxel Max brush palette (`brush.brushes[]`). Externally-tagged
/// enum: each entry is a single-key dict naming the slot type, e.g. `C(_)`
/// round-trips as `{"c": ...}`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum VMaxBrushEntry {
    /// Cube/voxel brush slot.
    C(VMaxBrushColor),

    /// `ch` brush slot.
    Ch(VMaxBrushColor),

    /// Ellipsoid/erase brush slot.
    E(VMaxBrushColor),

    /// `eh` brush slot.
    Eh(VMaxBrushColor),

    /// `bb` brush slot.
    Bb(VMaxBrushColor),

    /// `db` brush slot.
    Db(VMaxBrushColor),

    /// Prism brush slot.
    Pr(VMaxBrushColor),

    /// Pyramid brush slot.
    Py(VMaxBrushColor),

    /// `et` brush slot.
    Et(VMaxBrushColor),
}
