/// A single decoded voxel in a Voxel Max object's model space (0..256 per axis).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Voxel {
    pub position: [i32; 3],

    /// Material slot index (0–7) selecting an entry in the object's material palette.
    pub material_idx: u8,

    /// Color index (1–255) into the object's color palette; 0 is reserved as empty.
    pub color_idx: u8,
}
