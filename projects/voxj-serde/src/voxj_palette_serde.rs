use crate::AttrValueSerde;
use serde::{Deserialize, Serialize};
use voxj::VoxjPalette;

/// Serde-compatible parity type for [`VoxjPalette`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VoxjPaletteSerde {
    pub attributes: Vec<String>,
    pub data: Vec<Vec<AttrValueSerde>>,
}

impl From<VoxjPalette> for VoxjPaletteSerde {
    fn from(v: VoxjPalette) -> Self {
        Self {
            attributes: v.attributes,
            data: v
                .data
                .into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
        }
    }
}

impl From<VoxjPaletteSerde> for VoxjPalette {
    fn from(v: VoxjPaletteSerde) -> Self {
        Self {
            attributes: v.attributes,
            data: v
                .data
                .into_iter()
                .map(|row| row.into_iter().map(Into::into).collect())
                .collect(),
        }
    }
}
