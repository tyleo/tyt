use crate::GltfExtImageSource;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Per-image state preserved in the `gltf` ext, keyed by the image's id.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtImage {
    /// Where the embedded bytes were loaded from, or `None` for an image over
    /// one of the document's files or one the document gained after the
    /// load, which the writer places by its options.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub source: Option<GltfExtImageSource>,

    /// The image's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The image's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,
}
