use crate::{VXBrushState, VXFlag, VXToolMode, VXViewBox};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The Voxel Max tool state (`.vmaxb` `tools`): the active brush/material indices
/// plus the per-tool mode dictionaries that record how each editor surface is
/// configured. Voxel Max requires this key to import the object, so `from-voxj`
/// restores it field-for-field. Modes vary by tool but share the same wrapper
/// shape ([`VXToolMode`]).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXTools {
    /// Brush size (`bs`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub bs: i64,
    /// Active material index (`mi`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mi: i64,
    /// Active brush index (`bi`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub bi: i64,
    /// Active layer token (`al`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub al: String,
    /// Active-tool/source index (`src`); present on some Voxel Max objects.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub src: Option<i64>,
    /// Selection-tool flag (`stf`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub stf: Option<VXFlag>,
    /// Mirror flag (`mr`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub mr: Option<VXFlag>,
    /// Symmetry flag (`st`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub st: Option<VXFlag>,
    /// View/edit partition box (`vp`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub vp: Option<VXViewBox>,
    /// Brush state (`bst`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub bst: Option<VXBrushState>,
    /// Color tool (`ct`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ct: Option<VXToolMode>,
    /// Color-copy tool (`ctc`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctc: Option<VXToolMode>,
    /// Color-erase tool (`cte`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cte: Option<VXToolMode>,
    /// Color-paint tool (`ctp`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctp: Option<VXToolMode>,
    /// Color-select tool (`cts`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cts: Option<VXToolMode>,
    /// Color-material tool (`ctm`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ctm: Option<VXToolMode>,
    /// Paint color-material tool (`pctm`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pctm: Option<VXToolMode>,
    /// Color-add tool (`cta`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub cta: Option<VXToolMode>,
    /// Draw-mode brush (`dm`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dm: Option<VXToolMode>,
    /// Draw-mode brush-b (`dmb`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dmb: Option<VXToolMode>,
    /// Draw-mode color (`dmc`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dmc: Option<VXToolMode>,
    /// Draw-mode layer (`dml`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dml: Option<VXToolMode>,
    /// Draw-mode select (`dms`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub dms: Option<VXToolMode>,
}
