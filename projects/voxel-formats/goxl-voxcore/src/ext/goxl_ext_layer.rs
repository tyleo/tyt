use crate::GoxlExtPlacement;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Per-layer provenance preserved in the `goxl` ext, keyed by the hierarchy
/// node the layer became.
///
/// A `.gox` layer assembles its volume from shared blocks stamped at positions,
/// from a clone of another layer, or from a procedural shape. The blocks become
/// native objects and the placements that stamp them are recorded in
/// [`placements`](Self::placements). This keeps the layer's metadata and the
/// clone or shape definition, so the layer rebuilds exactly. The name comes
/// from the node.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GoxlExtLayer {
    /// Unique id within the file, referenced by a cloning layer's `base-id`.
    pub id: i32,

    /// Id of the layer this one clones, or `0` when it is not a clone.
    #[cfg_attr(feature = "serde", serde(rename = "base-id"))]
    pub base_id: i32,

    /// Index into the ext materials of the layer's material.
    pub material: i32,

    /// Volume blending mode.
    pub mode: i32,

    /// Whether the layer is visible.
    pub visible: bool,

    /// `mat`: the `4 x 4` transform applied to the layer.
    pub transform: [[f32; 4]; 4],

    /// `box`: the optional `4 x 4` edit box the author set.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "bounding-box",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub bounding_box: Option<[[f32; 4]; 4]>,

    /// `img-path`: the source image path for a 2D image layer, if any.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "image-path",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub image_path: Option<String>,

    /// `shape`: the procedural shape name for a shape layer, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub shape: Option<String>,

    /// `color`: the `[r, g, b, a]` color for a shape layer, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub color: Option<[u8; 4]>,

    /// The placed blocks, in stored order. The same object may be stamped
    /// at several positions. The distinct objects are the node's child
    /// objects. Empty for clone and shape layers.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub placements: Vec<GoxlExtPlacement>,

    /// Any further layer-dictionary keys, preserved verbatim as raw bytes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub extra: Vec<(String, Vec<u8>)>,
}
