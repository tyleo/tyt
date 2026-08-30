use crate::{VMaxBrushState, VMaxFlag, VMaxToolMode, VMaxViewBox};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel Max tool state (`.vmaxb` `tools`). Per-tool modes share the
/// [`VMaxToolMode`] wrapper shape.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxTools {
    /// Brush size.
    #[cfg_attr(feature = "serde", serde(default))]
    pub bs: i64,

    /// Active material index.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mi: i64,

    /// Active brush index.
    #[cfg_attr(feature = "serde", serde(default))]
    pub bi: i64,

    /// Active layer token.
    #[cfg_attr(feature = "serde", serde(default))]
    pub al: String,

    /// Active-tool/source index; present on some objects.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub src: Option<i64>,

    /// Selection-tool [`VMaxFlag`](crate::VMaxFlag).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub stf: Option<VMaxFlag>,

    /// Mirror [`VMaxFlag`](crate::VMaxFlag).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub mr: Option<VMaxFlag>,

    /// Symmetry [`VMaxFlag`](crate::VMaxFlag).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub st: Option<VMaxFlag>,

    /// View/edit partition box ([`VMaxViewBox`](crate::VMaxViewBox)).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub vp: Option<VMaxViewBox>,

    /// [`VMaxBrushState`](crate::VMaxBrushState).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub bst: Option<VMaxBrushState>,

    /// Color tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ct: Option<VMaxToolMode>,

    /// Color-copy tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctc: Option<VMaxToolMode>,

    /// Color-erase tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cte: Option<VMaxToolMode>,

    /// Color-paint tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctp: Option<VMaxToolMode>,

    /// Color-select tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cts: Option<VMaxToolMode>,

    /// Color-material tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctm: Option<VMaxToolMode>,

    /// Paint color-material tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pctm: Option<VMaxToolMode>,

    /// Color-add tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cta: Option<VMaxToolMode>,

    /// Draw-mode brush.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dm: Option<VMaxToolMode>,

    /// Draw-mode brush-b.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dmb: Option<VMaxToolMode>,

    /// Draw-mode color.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dmc: Option<VMaxToolMode>,

    /// Draw-mode layer.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dml: Option<VMaxToolMode>,

    /// Draw-mode select.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dms: Option<VMaxToolMode>,
}
