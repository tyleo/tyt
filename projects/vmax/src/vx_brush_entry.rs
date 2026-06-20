use crate::VXBrushColor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One slot in a Voxel Max brush palette (`brush.brushes[]`): a single-key
/// dictionary tagging the slot type, whose value is that slot's color payload.
/// Voxel Max emits exactly one key per entry, drawn from a fixed set, so the slot
/// type is an externally-tagged enum — `C(_)` round-trips as `{"c": …}`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum VXBrushEntry {
    /// Cube/voxel brush slot (`c`).
    C(VXBrushColor),
    /// `ch` brush slot.
    Ch(VXBrushColor),
    /// Ellipsoid/erase brush slot (`e`).
    E(VXBrushColor),
    /// `eh` brush slot.
    Eh(VXBrushColor),
    /// `bb` brush slot.
    Bb(VXBrushColor),
    /// `db` brush slot.
    Db(VXBrushColor),
    /// Prism brush slot (`pr`).
    Pr(VXBrushColor),
    /// Pyramid brush slot (`py`).
    Py(VXBrushColor),
    /// `et` brush slot.
    Et(VXBrushColor),
}
