use crate::{SdfjMap, SdfjPropertyValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A material: an entry of [`SdfjFile::materials`](crate::SdfjFile::materials).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase",
        deny_unknown_fields
    )
)]
pub enum SdfjMaterial {
    /// A material holding its properties by name.
    Material {
        properties: SdfjMap<SdfjPropertyValue>,
    },

    /// The shade at `index` of the `shades` entry, counting from the darkest.
    Shade { shades: usize, index: usize },
}
