use serde::{Deserialize, Serialize};

/// Per-material provenance preserved in the `magica-voxel` ext. A material's
/// type and scalar fields fold into the palette cell named by [`id`](Self::id);
/// this records that the material exists, in its stored order, and keeps the
/// arbitrary `extra` keys that do not fit a palette attribute.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MagicaVoxelMaterial {
    /// The material id, which is the palette index it folds into.
    pub id: i32,

    /// Any further property keys, preserved verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String)>,
}
