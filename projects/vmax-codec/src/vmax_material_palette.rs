use crate::VMaxMaterial;

/// A Voxel Max material palette decoded from a `palette*.settings.vmaxpsb`: the
/// selectable material slots plus the palette's display name.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxMaterialPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,
    /// The selectable material slots.
    pub materials: Vec<VMaxMaterial>,
}
