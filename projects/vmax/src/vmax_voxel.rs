/// A single decoded voxel in a Voxel Max object's model space (0..256 per axis).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VMaxVoxel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// Material slot index (0–7) selecting an entry in the object's material palette.
    pub material: u8,
    /// Color index (1–255) into the object's color palette; 0 is reserved as empty.
    pub color: u8,
}
