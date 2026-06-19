use crate::VXBrushEntry;
use serde::{Deserialize, Serialize};

/// The Voxel Max brush palette (`.vmaxb` `brush`): a named set of brush slots and
/// the currently selected slot. Stored so `from-voxj` restores it; Voxel Max
/// requires the key to import the object.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VXBrush {
    /// Palette display name (`name`).
    pub name: String,
    /// Brush slots (`brushes`).
    #[serde(default)]
    pub brushes: Vec<VXBrushEntry>,
    /// Index of the selected slot (`current`).
    pub current: i64,
}
