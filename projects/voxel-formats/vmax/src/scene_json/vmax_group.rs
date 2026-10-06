use crate::VMaxValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A group node in a Voxel Max scene hierarchy.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxGroup {
    /// Display name.
    #[cfg_attr(feature = "serde", serde(default))]
    pub name: String,

    /// Node id.
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: String,

    /// Parent group id (`pid`), or `None` at the root.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "pid", skip_serializing_if = "Option::is_none", default)
    )]
    pub parent_id: Option<String>,

    /// Hidden flag; present on some nodes.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "h", skip_serializing_if = "Option::is_none", default)
    )]
    pub hidden: Option<bool>,

    /// Position (`t_p`).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "t_p", serialize_with = "crate::finite")
    )]
    pub position: [f64; 3],

    /// Rotation as an `[x, y, z, angle]` axis-angle (`t_r`).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "t_r", serialize_with = "crate::finite")
    )]
    pub rotation: [f64; 4],

    /// Scale (`t_s`).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "t_s", serialize_with = "crate::finite")
    )]
    pub scale: [f64; 3],

    /// Hierarchy sort/path triple.
    #[cfg_attr(feature = "serde", serde(default))]
    pub ind: [i64; 3],

    /// Selection/visibility flag; present on some nodes.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub s: Option<bool>,

    /// Transform-anchor token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub t_al: String,

    /// Transform pivot-axis token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub t_pa: String,

    /// Transform pivot-face token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub t_pf: String,

    /// Transform pivot-offset; shape varies, kept as untyped [`VMaxValue`]
    /// (round-trips unchanged).
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            serialize_with = "crate::json_value"
        )
    )]
    pub t_po: Option<VMaxValue>,

    /// Center of the group's voxel bounds in model space.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "e_c", default, serialize_with = "crate::finite")
    )]
    pub center: [f64; 3],

    /// Min corner of the group's voxel bounds, relative to
    /// [`center`](Self::center).
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "e_mi",
            skip_serializing_if = "Option::is_none",
            default,
            serialize_with = "crate::finite"
        )
    )]
    pub bounds_min: Option<[f64; 3]>,

    /// Max corner of the group's voxel bounds, relative to
    /// [`center`](Self::center).
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "e_ma",
            skip_serializing_if = "Option::is_none",
            default,
            serialize_with = "crate::finite"
        )
    )]
    pub bounds_max: Option<[f64; 3]>,

    /// The pivot reference an imported joint fixes its pivot to (`t_prp`):
    /// editor state, which a scene rebuilt from voxels leaves out.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            serialize_with = "crate::finite"
        )
    )]
    pub t_prp: Option<[f64; 3]>,

    /// The cached center of mass (`e_cm`), which Voxel Max recomputes after a
    /// load.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            serialize_with = "crate::finite"
        )
    )]
    pub e_cm: Option<[f64; 3]>,

    /// The version of the cached center of mass (`e_cmv`). Voxel Max honors
    /// `e_cm` only at version 2.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub e_cmv: Option<i64>,

    /// The cached voxel count (`e_vc`), which Voxel Max recomputes after a
    /// load.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub e_vc: Option<i64>,

    /// The cached mass (`e_vm`), present when it differs from the voxel
    /// count.
    #[cfg_attr(
        feature = "serde",
        serde(
            skip_serializing_if = "Option::is_none",
            default,
            serialize_with = "crate::finite"
        )
    )]
    pub e_vm: Option<f64>,
}
