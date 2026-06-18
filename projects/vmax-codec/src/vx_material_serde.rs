use serde::Deserialize;
use vmax::VMaxMaterial;

/// A single material slot decoded from a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
pub struct VXMaterialSerde {
    /// Metalness coefficient.
    #[serde(default)]
    pub mc: f64,
    /// Roughness coefficient.
    #[serde(default)]
    pub rc: f64,
    /// Self-illumination (emission) coefficient.
    #[serde(default)]
    pub sic: f64,
    /// Whether the material casts shadows.
    #[serde(default)]
    pub sh: bool,
}

impl From<VXMaterialSerde> for VMaxMaterial {
    fn from(v: VXMaterialSerde) -> Self {
        Self {
            metalness: v.mc,
            roughness: v.rc,
            emission: v.sic,
            enable_shadows: v.sh,
        }
    }
}
