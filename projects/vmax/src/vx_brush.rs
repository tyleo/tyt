use crate::VXBrushEntry;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The Voxel Max brush palette: a named set of brush slots and the currently
/// selected slot.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXBrush {
    /// Palette display name.
    pub name: String,
    /// Brush slots.
    #[cfg_attr(feature = "serde", serde(default))]
    pub brushes: Vec<VXBrushEntry>,
    /// Index of the selected slot.
    pub current: i64,
}
