use vmax::VXMaterial;

/// A Voxel Max material slot (one of the eight selectable per palette).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VMaxMaterial {
    /// Metalness in `0..=1` (Voxel Max `mc`).
    pub metalness: f64,
    /// Roughness in `0..=1` (Voxel Max `rc`).
    pub roughness: f64,
    /// Emission strength (Voxel Max `sic`).
    pub emission: f64,
    /// Whether the material casts shadows (Voxel Max `sh`).
    pub enable_shadows: bool,
}

impl From<VXMaterial> for VMaxMaterial {
    fn from(v: VXMaterial) -> Self {
        Self {
            metalness: v.mc,
            roughness: v.rc,
            emission: v.sic,
            enable_shadows: v.sh,
        }
    }
}
