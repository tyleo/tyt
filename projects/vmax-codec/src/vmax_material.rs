use crate::VMaxDispersion;
use vmax::VMaxMaterial as VMaxMaterialRaw;

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

    /// Optional dispersion parameters (Voxel Max `md`); `None` when the slot
    /// carries no `md` block.
    pub dispersion: Option<VMaxDispersion>,
}

impl From<VMaxMaterialRaw> for VMaxMaterial {
    fn from(v: VMaxMaterialRaw) -> Self {
        Self {
            metalness: v.mc,
            roughness: v.rc,
            emission: v.sic,
            enable_shadows: v.sh,
            dispersion: v.md.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{VMaxDispersion, VMaxMaterial};
    use vmax::{VMaxMaterial as VMaxMaterialRaw, VMaxMaterialMd};

    #[test]
    fn maps_md_to_dispersion() {
        let v = VMaxMaterialRaw {
            mi: "1".to_owned(),
            mc: 0.66,
            rc: 0.58,
            sic: 4.2,
            sh: true,
            md: Some(VMaxMaterialMd {
                a: 0.0,
                i: 1.5,
                t: 0.83,
            }),
        };
        assert_eq!(
            VMaxMaterial::from(v).dispersion,
            Some(VMaxDispersion {
                absorption: 0.0,
                ior: 1.5,
                transmission: 0.83,
            })
        );
    }

    #[test]
    fn absent_md_is_no_dispersion() {
        let v = VMaxMaterialRaw {
            mi: "1".to_owned(),
            mc: 0.1,
            rc: 0.9,
            sic: 0.0,
            sh: true,
            md: None,
        };
        assert_eq!(VMaxMaterial::from(v).dispersion, None);
    }
}
