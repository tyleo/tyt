use crate::{VXBrushState, VXFlag, VXToolMode, VXViewBox};
use serde::{Deserialize, Serialize};

/// The Voxel Max tool state (`.vmaxb` `tools`): the active brush/material indices
/// plus the per-tool mode dictionaries that record how each editor surface is
/// configured. Voxel Max requires this key to import the object, so `from-voxj`
/// restores it field-for-field. Modes vary by tool but share the same wrapper
/// shape ([`VXToolMode`]).
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXTools {
    /// Brush size (`bs`).
    #[serde(default)]
    pub bs: i64,
    /// Active material index (`mi`).
    #[serde(default)]
    pub mi: i64,
    /// Active brush index (`bi`).
    #[serde(default)]
    pub bi: i64,
    /// Active layer token (`al`).
    #[serde(default)]
    pub al: String,
    /// Selection-tool flag (`stf`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stf: Option<VXFlag>,
    /// Mirror flag (`mr`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mr: Option<VXFlag>,
    /// Symmetry flag (`st`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub st: Option<VXFlag>,
    /// View/edit partition box (`vp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vp: Option<VXViewBox>,
    /// Brush state (`bst`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bst: Option<VXBrushState>,
    /// Color tool (`ct`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ct: Option<VXToolMode>,
    /// Color-copy tool (`ctc`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctc: Option<VXToolMode>,
    /// Color-erase tool (`cte`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cte: Option<VXToolMode>,
    /// Color-paint tool (`ctp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctp: Option<VXToolMode>,
    /// Color-select tool (`cts`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cts: Option<VXToolMode>,
    /// Color-material tool (`ctm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctm: Option<VXToolMode>,
    /// Paint color-material tool (`pctm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pctm: Option<VXToolMode>,
    /// Color-add tool (`cta`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cta: Option<VXToolMode>,
    /// Draw-mode brush (`dm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dm: Option<VXToolMode>,
    /// Draw-mode brush-b (`dmb`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dmb: Option<VXToolMode>,
    /// Draw-mode color (`dmc`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dmc: Option<VXToolMode>,
    /// Draw-mode layer (`dml`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dml: Option<VXToolMode>,
    /// Draw-mode select (`dms`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dms: Option<VXToolMode>,
}
