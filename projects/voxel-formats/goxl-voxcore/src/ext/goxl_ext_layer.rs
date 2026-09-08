#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// Per-layer provenance preserved in the `goxl` ext, aligned by index with the
/// hierarchy nodes, in stored order.
///
/// A `.gox` layer assembles its volume from shared blocks stamped at positions,
/// from a clone of another layer, or from a procedural shape. The blocks become
/// native objects and the placements that stamp them are recorded in
/// [`placements`](Self::placements); this struct keeps the layer's metadata and
/// the clone/shape definition so the layer rebuilds exactly.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxlExtLayer {
    /// Layer name.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "String::is_empty")
    )]
    pub name: String,

    /// Unique id within the file, referenced by a cloning layer's `base-id`.
    pub id: i32,

    /// Id of the layer this one clones, or `0` when it is not a clone.
    #[cfg_attr(feature = "ext", serde(rename = "base-id"))]
    pub base_id: i32,

    /// Index into the ext materials of the layer's material.
    pub material: i32,

    /// Volume blending mode.
    pub mode: i32,

    /// Whether the layer is visible.
    pub visible: bool,

    /// `mat`: the `4 x 4` transform applied to the layer.
    pub transform: [[f32; 4]; 4],

    /// `box`: the optional `4 x 4` bounding box.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "bounding-box",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub bounding_box: Option<[[f32; 4]; 4]>,

    /// `img-path`: the source image path for a 2D image layer, if any.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "image-path",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub image_path: Option<String>,

    /// `shape`: the procedural shape name for a shape layer, if any.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub shape: Option<String>,

    /// `color`: the `[r, g, b, a]` color for a shape layer, if any.
    #[cfg_attr(
        feature = "ext",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub color: Option<[u8; 4]>,

    /// The placed blocks, in stored order, as `(block index, [x, y, z])`. The
    /// block index names a native object; the same block may be stamped at
    /// several positions. Empty for clone and shape layers.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub placements: Vec<(i32, [i32; 3])>,

    /// Any further layer-dictionary keys, preserved verbatim as raw bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, Vec<u8>)>,
}
