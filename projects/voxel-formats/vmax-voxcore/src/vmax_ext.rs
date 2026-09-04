use crate::{VMaxExtNode, VMaxExtObjectState, VMaxExtPalette};
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};
use vmax::VMaxSceneJsonFile;

/// The `vmax` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Voxel Max state with no native voxcore home, kept so a document loaded from
/// a Voxel Max package can be written back exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct VMaxExt {
    /// The Voxel Max scene with its hierarchy emptied, holding only the
    /// scene-level state recorded around the objects and groups that voxcore
    /// represents natively.
    pub scene: VMaxSceneJsonFile,

    /// Per-node provenance, aligned by index with the hierarchy nodes, groups
    /// before objects.
    #[cfg_attr(feature = "ext", serde(rename = "hierarchy-nodes"))]
    pub hierarchy_nodes: Vec<VMaxExtNode>,

    /// Per-palette provenance, aligned by index with the palettes. A material
    /// palette holds its name; a color palette holds nothing.
    pub palettes: Vec<Option<VMaxExtPalette>>,

    /// Per-object editor state, aligned by index with the objects. An object
    /// with no `contents*.vmaxb` holds nothing.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "object-states",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub object_states: Vec<Option<VMaxExtObjectState>>,
}
