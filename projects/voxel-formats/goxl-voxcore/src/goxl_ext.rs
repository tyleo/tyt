use crate::{
    GoxlExtCamera, GoxlExtImage, GoxlExtLayer, GoxlExtLight, GoxlExtMaterial, GoxlExtPreview,
    GoxlExtUnknownChunk,
};
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};
use std::any::Any;
#[cfg(not(feature = "ext"))]
use voxcore::ext::Error;
#[cfg(feature = "ext")]
use voxcore::ext::encode_entry;
use voxcore::{
    VoxMap,
    ext::{Result, VoxExt},
};

/// The `goxl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Goxel `.gox` state with no native voxcore home, kept so a file loaded from a
/// Goxel package can be written back exactly.
///
/// The shared `BL16` voxel blocks become native objects and the per-layer block
/// placements that stamp them become the hierarchy nodes; this holds the rest,
/// with the per-layer entries aligned by index with the hierarchy nodes so the
/// file rebuilds exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxlExt {
    /// The format version from the header.
    pub version: i32,

    /// The `IMG ` image metadata.
    #[cfg_attr(feature = "ext", serde(default))]
    pub image: GoxlExtImage,

    /// The `PREV` preview thumbnail, or `None` when the file omits it.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub preview: Option<GoxlExtPreview>,

    /// The `MATE` materials, in stored order; a layer names one by index.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub materials: Vec<GoxlExtMaterial>,

    /// Per-layer provenance, aligned by index with the hierarchy nodes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub layers: Vec<GoxlExtLayer>,

    /// The `CAMR` cameras, in stored order.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub cameras: Vec<GoxlExtCamera>,

    /// The `LIGH` light settings, or `None` when the file omits them.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub light: Option<GoxlExtLight>,

    /// Chunks the goxl crate does not model, preserved verbatim.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "unknown-chunks",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub unknown_chunks: Vec<GoxlExtUnknownChunk>,
}

/// The Goxel ext as a state's ext. Its block is the `goxl` entry.
/// Encoding it needs the `ext` feature.
impl VoxExt for GoxlExt {
    #[cfg(feature = "ext")]
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    #[cfg(not(feature = "ext"))]
    fn to_vox_ext(&self) -> Result<VoxMap> {
        Err(Error::Invalid(
            "the Goxel ext encodes its block only with the `ext` feature".to_owned(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }
}
