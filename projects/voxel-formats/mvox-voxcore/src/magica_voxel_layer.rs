#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// A layer (`LAYR`) preserved verbatim in the `magica-voxel` ext: a named,
/// optionally hidden grouping transform nodes assign themselves to by id.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct MagicaVoxelLayer {
    /// The layer id transform nodes reference.
    pub id: i32,

    /// `_name`: the layer's display name.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,

    /// `_hidden`: whether the layer is hidden.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub hidden: Option<bool>,

    /// Any further attribute keys, preserved verbatim.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, String)>,
}
