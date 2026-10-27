#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The `asset` block preserved in the `gltf` ext. It has no native meshdoc
/// home, so it rides here verbatim.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExtAsset {
    /// The glTF version, `2.0`.
    pub version: String,

    /// The minimum glTF version needed, if declared.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "min-version",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub min_version: Option<String>,

    /// The tool that wrote the document, if declared.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub generator: Option<String>,

    /// The copyright notice, if declared.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub copyright: Option<String>,

    /// The block's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,
}

impl Default for GltfExtAsset {
    fn default() -> Self {
        Self {
            version: "2.0".to_owned(),
            min_version: None,
            generator: None,
            copyright: None,
            extras: None,
        }
    }
}
