use crate::{VXBrushState, VXFlag, VXToolMode, VXViewBox};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The Voxel Max tool state (`.vmaxb` `tools`): the active brush/material
/// indices plus the per-tool mode dictionaries that record how each editor
/// surface is configured. Voxel Max requires this key to import the object.
/// Modes vary by tool but share the same wrapper shape ([`VXToolMode`]).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXTools {
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
    /// Active-tool/source index; present on some Voxel Max objects.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub src: Option<i64>,
    /// Selection-tool [`VXFlag`](crate::VXFlag).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub stf: Option<VXFlag>,
    /// Mirror [`VXFlag`](crate::VXFlag).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub mr: Option<VXFlag>,
    /// Symmetry [`VXFlag`](crate::VXFlag).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub st: Option<VXFlag>,
    /// View/edit partition box ([`VXViewBox`](crate::VXViewBox)).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub vp: Option<VXViewBox>,
    /// [`VXBrushState`](crate::VXBrushState).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub bst: Option<VXBrushState>,
    /// Color tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ct: Option<VXToolMode>,
    /// Color-copy tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctc: Option<VXToolMode>,
    /// Color-erase tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cte: Option<VXToolMode>,
    /// Color-paint tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctp: Option<VXToolMode>,
    /// Color-select tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cts: Option<VXToolMode>,
    /// Color-material tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctm: Option<VXToolMode>,
    /// Paint color-material tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pctm: Option<VXToolMode>,
    /// Color-add tool.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cta: Option<VXToolMode>,
    /// Draw-mode brush.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dm: Option<VXToolMode>,
    /// Draw-mode brush-b.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dmb: Option<VXToolMode>,
    /// Draw-mode color.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dmc: Option<VXToolMode>,
    /// Draw-mode layer.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dml: Option<VXToolMode>,
    /// Draw-mode select.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dms: Option<VXToolMode>,
}
