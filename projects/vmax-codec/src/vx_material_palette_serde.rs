use crate::VXMaterialSerde;
use serde::Deserialize;
use vmax::VMaxMaterialPalette;

/// Material palette decoded from a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct VXMaterialPaletteSerde {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub materials: Vec<VXMaterialSerde>,
}

impl VXMaterialPaletteSerde {
    /// Returns the decoded palette as a core [`VMaxMaterialPalette`].
    pub fn palette(&self) -> VMaxMaterialPalette {
        VMaxMaterialPalette {
            name: self.name.clone(),
            materials: self.materials.iter().copied().map(Into::into).collect(),
        }
    }
}
