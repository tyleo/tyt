#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The `IMG ` image-metadata chunk preserved in the `goxl` ext. The geometry
/// and colors live in native objects, so this keeps only the file-level image
/// metadata.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct GoxlExtImage {
    /// `box`: the optional `4 x 4` bounding box of the whole image.
    #[cfg_attr(
        feature = "ext",
        serde(
            rename = "bounding-box",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub bounding_box: Option<[[f32; 4]; 4]>,

    /// Any further image-dictionary keys, preserved verbatim as raw bytes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub extra: Vec<(String, Vec<u8>)>,
}
