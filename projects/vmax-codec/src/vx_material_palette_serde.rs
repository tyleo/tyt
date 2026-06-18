use crate::VXMaterialSerde;
use serde::Deserialize;
use vmax::VMaxMaterial;

/// Material palette decoded from a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct VXMaterialPaletteSerde {
    #[serde(default)]
    pub materials: Vec<VXMaterialSerde>,
}

impl VXMaterialPaletteSerde {
    /// Returns the material slots as core [`VMaxMaterial`] values.
    pub fn materials(&self) -> Vec<VMaxMaterial> {
        self.materials.iter().copied().map(Into::into).collect()
    }
}
