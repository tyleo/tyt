use crate::{VMaxMaterialDispersion, VMaxSerdeMaterial};

/// A Voxel Max material slot (one of the eight selectable per palette).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VMaxCodecMaterial {
    /// Metalness in `0..=1` (Voxel Max `mc`).
    pub metalness: f64,

    /// Roughness in `0..=1` (Voxel Max `rc`).
    pub roughness: f64,

    /// Emission strength (Voxel Max `sic`).
    pub emission: f64,

    /// Whether the material casts shadows (Voxel Max `sh`).
    pub enable_shadows: bool,

    /// Optional dispersion parameters (Voxel Max `md`); `None` when the slot
    /// carries no `md` block.
    pub dispersion: Option<VMaxMaterialDispersion>,
}

impl From<VMaxSerdeMaterial> for VMaxCodecMaterial {
    fn from(v: VMaxSerdeMaterial) -> Self {
        Self {
            metalness: v.mc,
            roughness: v.rc,
            emission: v.sic,
            enable_shadows: v.sh,
            dispersion: v.md,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{VMaxCodecMaterial, VMaxMaterialDispersion, VMaxSerdeMaterial};

    #[test]
    fn maps_md_to_dispersion() {
        let v = VMaxSerdeMaterial {
            mi: "1".to_owned(),
            mc: 0.66,
            rc: 0.58,
            sic: 4.2,
            sh: true,
            md: Some(VMaxMaterialDispersion {
                absorption: 0.0,
                ior: 1.5,
                transmission: 0.83,
            }),
        };
        assert_eq!(
            VMaxCodecMaterial::from(v).dispersion,
            Some(VMaxMaterialDispersion {
                absorption: 0.0,
                ior: 1.5,
                transmission: 0.83,
            })
        );
    }

    #[test]
    fn absent_md_is_no_dispersion() {
        let v = VMaxSerdeMaterial {
            mi: "1".to_owned(),
            mc: 0.1,
            rc: 0.9,
            sic: 0.0,
            sh: true,
            md: None,
        };
        assert_eq!(VMaxCodecMaterial::from(v).dispersion, None);
    }
}
