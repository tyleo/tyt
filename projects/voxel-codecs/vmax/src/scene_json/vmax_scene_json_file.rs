use crate::{VMaxGroup, VMaxObject, VMaxSceneCamera};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A complete Voxel Max scene parsed from `scene.json`.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxSceneJsonFile {
    /// Group nodes (hierarchy folders).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub groups: Vec<VMaxGroup>,

    /// Object nodes (voxel models). Always serialized, even when empty: Voxel
    /// Max writes `objects: []` on every document and rejects one that omits
    /// the key, so an object-less scene must keep it (unlike `groups`, which it
    /// omits when empty).
    pub objects: Vec<VMaxObject>,

    /// Codable scene version.
    #[cfg_attr(feature = "serde", serde(default = "default_scene_version"))]
    pub v: i64,

    /// Scene camera / light rig.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cam: Option<VMaxSceneCamera>,

    /// Antialiasing flag, e.g. `"t"`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub af: Option<String>,

    /// Antialiasing quality level, e.g. `2`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ag: Option<i64>,

    /// Ambient-light intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aint: Option<f64>,

    /// Ambient-occlusion amount.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ao: Option<f64>,

    /// Background color, e.g. `"#151313FF"`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub background: Option<String>,

    /// Bloom blur radius.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bloombrad: Option<f64>,

    /// Bloom intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bloomint: Option<f64>,

    /// Bloom threshold.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bloomthr: Option<f64>,

    /// Contrast.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cont: Option<f64>,

    /// Exposure / environment intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub eint: Option<f64>,

    /// Film-grain intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub graint: Option<f64>,

    /// Key-light color, e.g. `"#FFFFFFFF"`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lcolor: Option<String>,

    /// Key-light intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lint: Option<f64>,

    /// Noise-reduction / denoise flag.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nrn: Option<bool>,

    /// Scene-level boolean flag present on some documents.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub oie: Option<bool>,

    /// Outline intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub outlineint: Option<f64>,

    /// Outline size.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub outlinesz: Option<f64>,

    /// Saturation.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub sat: Option<f64>,

    /// Shadow intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub shadowint: Option<f64>,

    /// Screen-space reflections enabled.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ssr: Option<bool>,

    /// Color temperature.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub temp: Option<f64>,

    /// Color tint.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tint: Option<f64>,

    /// Vignette intensity.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub vigint: Option<f64>,

    /// Vignette falloff power.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub vigpow: Option<f64>,
}

/// Current codable scene version; the `v` fallback when the key is absent.
#[cfg(feature = "serde")]
fn default_scene_version() -> i64 {
    4
}
