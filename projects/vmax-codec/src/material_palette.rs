use crate::Material;

/// A Voxel Max material palette decoded from a `palette*.settings.vmaxpsb`: the
/// selectable material slots, the palette's display name, and the RGBA color
/// table.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialPalette {
    /// Display name (Voxel Max `name`).
    pub name: String,

    /// The selectable material slots.
    pub materials: Vec<Material>,

    /// One `[r, g, b, a]` per color index. This is Voxel Max's color source
    /// when an object's `palette*.png` image is absent; empty when the palette
    /// carries no color table.
    pub colors: Vec<[u8; 4]>,
}
