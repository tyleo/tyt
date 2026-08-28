use serde::{Deserialize, Serialize};

/// The `IMG ` image-metadata chunk preserved in the `goxel` ext. The geometry
/// and colors live in native objects, so this keeps only the file-level image
/// metadata.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GoxelImage {
    /// `box`: the optional `4 x 4` bounding box of the whole image.
    #[serde(
        rename = "bounding-box",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub bounding_box: Option<[[f32; 4]; 4]>,

    /// Any further image-dictionary keys, preserved verbatim as raw bytes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, Vec<u8>)>,
}
