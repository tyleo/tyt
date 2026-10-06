use crate::{VMaxGroup, VMaxObject, VMaxSceneCamera, VMaxValue};
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

    /// The face the scene camera's view snaps to: `"f"`, `"r"`, `"ba"`,
    /// `"l"`, `"t"`, or `"bt"`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub af: Option<String>,

    /// The active group, an index into `groups`. Voxel Max reads it unchecked,
    /// so it must name a listed group.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ag: Option<i64>,

    /// Ambient-light intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub aint: Option<f64>,

    /// The active object, an index into `objects`. Voxel Max reads it
    /// unchecked, so it must name a listed object.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ao: Option<i64>,

    /// Background color, e.g. `"#151313FF"`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub background: Option<String>,

    /// Bloom blur radius.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub bloombrad: Option<f64>,

    /// Bloom intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub bloomint: Option<f64>,

    /// Bloom threshold.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub bloomthr: Option<f64>,

    /// Contrast.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub cont: Option<f64>,

    /// Exposure / environment intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub eint: Option<f64>,

    /// Film-grain intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub graint: Option<f64>,

    /// Key-light color, e.g. `"#FFFFFFFF"`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lcolor: Option<String>,

    /// Key-light intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub lint: Option<f64>,

    /// Noise-reduction / denoise flag.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nrn: Option<bool>,

    /// Whether the scene holds no object on purpose, which keeps Voxel Max
    /// from rebuilding the objects from the scene's history.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub oie: Option<bool>,

    /// The level the scene opens on: `-1` for the scene, else an index into
    /// `groups`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub vl: Option<i64>,

    /// The textures each external mesh's contents file references, as stored.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub trsc: Option<VMaxValue>,

    /// Outline intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub outlineint: Option<f64>,

    /// Outline size.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub outlinesz: Option<f64>,

    /// Saturation.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub sat: Option<f64>,

    /// Shadow intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub shadowint: Option<f64>,

    /// Screen-space reflections enabled.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ssr: Option<bool>,

    /// Color temperature.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub temp: Option<f64>,

    /// Color tint.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub tint: Option<f64>,

    /// Vignette intensity.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub vigint: Option<f64>,

    /// Vignette falloff power.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "crate::finite"
        )
    )]
    pub vigpow: Option<f64>,
}

/// Current codable scene version; the `v` fallback when the key is absent.
#[cfg(feature = "serde")]
fn default_scene_version() -> i64 {
    4
}
