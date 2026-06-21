use vmax::VXMaterialMd;

/// Decoded dispersion parameters for a material slot — Voxel Max's optional `md`
/// block: absorption, index of refraction, and transmission. Present only when
/// the slot's advanced material has been edited; the values may all be defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VMaxDispersion {
    /// Absorption (Voxel Max `md.a`).
    pub absorption: f64,

    /// Index of refraction (Voxel Max `md.i`).
    pub ior: f64,

    /// Light transmission through the surface (Voxel Max `md.t`).
    pub transmission: f64,
}

impl From<VXMaterialMd> for VMaxDispersion {
    fn from(v: VXMaterialMd) -> Self {
        Self {
            absorption: v.a,
            ior: v.i,
            transmission: v.t,
        }
    }
}
