use crate::{VMaxGroup, VMaxObject, VMaxSceneCamera};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A complete Voxel Max scene parsed from `scene.json`: the `groups`/`objects`
/// hierarchy plus the scene-level state Voxel Max records around it: the
/// codable version, the camera/light rig, and the renderer / post-grading / UI
/// settings.
///
/// Every scene-level setting except the required version `v` is `Option` and
/// skipped when `None`, so a scene that omits a key round-trips without it being
/// re-introduced (Voxel Max tolerates the absence and backfills its own default
/// on open). `v` is the one exception: Voxel Max will not open a scene that lacks
/// it, so `v` is never skipped and always emitted, defaulting to the current
/// codable version when a decoded scene omits it. A key Voxel Max stores as
/// either an integer or a real (the grading sliders) is accepted as a real either
/// way. Whole-number sliders therefore re-serialize as reals (`0.0`), which Voxel
/// Max's JSON decoder reads back into its numeric type unchanged.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxScene {
    /// Group nodes (hierarchy folders).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub groups: Vec<VMaxGroup>,
    /// Object nodes (voxel models).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
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

/// The current Voxel Max codable scene version, used as the `v` fallback when a
/// decoded scene omits the key Voxel Max requires.
#[cfg(feature = "serde")]
fn default_scene_version() -> i64 {
    4
}
