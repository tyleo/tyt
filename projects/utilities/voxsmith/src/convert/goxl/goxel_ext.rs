use crate::{
    GoxelCamera, GoxelImage, GoxelLayer, GoxelLight, GoxelMaterial, GoxelPreview, GoxelUnknownChunk,
};
use serde::{Deserialize, Serialize};

/// The `goxel` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Goxel `.gox` state with no native voxcore home, kept so a file loaded from a
/// Goxel package can be written back exactly.
///
/// The shared `BL16` voxel blocks become native objects and the per-layer block
/// placements that stamp them become the hierarchy nodes; this holds the rest,
/// with the per-layer entries aligned by index with the hierarchy nodes so the
/// file rebuilds exactly.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GoxelExt {
    /// The format version from the header.
    pub version: i32,

    /// The `IMG ` image metadata.
    #[serde(default)]
    pub image: GoxelImage,

    /// The `PREV` preview thumbnail, or `None` when the file omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<GoxelPreview>,

    /// The `MATE` materials, in stored order; a layer names one by index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<GoxelMaterial>,

    /// Per-layer provenance, aligned by index with the hierarchy nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<GoxelLayer>,

    /// The `CAMR` cameras, in stored order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cameras: Vec<GoxelCamera>,

    /// The `LIGH` light settings, or `None` when the file omits them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light: Option<GoxelLight>,

    /// Chunks the goxl crate does not model, preserved verbatim.
    #[serde(
        rename = "unknown-chunks",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub unknown_chunks: Vec<GoxelUnknownChunk>,
}
