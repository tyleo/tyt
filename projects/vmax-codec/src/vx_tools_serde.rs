use crate::{VXBrushStateSerde, VXFlagSerde, VXToolModeSerde, VXViewBoxSerde};
use serde::{Deserialize, Serialize};

/// The Voxel Max tool state (`.vmaxb` `tools`): the active brush/material indices
/// plus the per-tool mode dictionaries that record how each editor surface is
/// configured. Voxel Max requires this key to import the object, so `from-voxj`
/// restores it field-for-field. Modes vary by tool but share the same wrapper
/// shape ([`VXToolModeSerde`]).
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXToolsSerde {
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
    pub stf: Option<VXFlagSerde>,
    /// Mirror flag (`mr`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mr: Option<VXFlagSerde>,
    /// Symmetry flag (`st`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub st: Option<VXFlagSerde>,
    /// View/edit partition box (`vp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vp: Option<VXViewBoxSerde>,
    /// Brush state (`bst`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bst: Option<VXBrushStateSerde>,
    /// Color tool (`ct`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ct: Option<VXToolModeSerde>,
    /// Color-copy tool (`ctc`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctc: Option<VXToolModeSerde>,
    /// Color-erase tool (`cte`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cte: Option<VXToolModeSerde>,
    /// Color-paint tool (`ctp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctp: Option<VXToolModeSerde>,
    /// Color-select tool (`cts`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cts: Option<VXToolModeSerde>,
    /// Color-material tool (`ctm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctm: Option<VXToolModeSerde>,
    /// Paint color-material tool (`pctm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pctm: Option<VXToolModeSerde>,
    /// Color-add tool (`cta`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cta: Option<VXToolModeSerde>,
    /// Draw-mode brush (`dm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dm: Option<VXToolModeSerde>,
    /// Draw-mode brush-b (`dmb`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dmb: Option<VXToolModeSerde>,
    /// Draw-mode color (`dmc`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dmc: Option<VXToolModeSerde>,
    /// Draw-mode layer (`dml`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dml: Option<VXToolModeSerde>,
    /// Draw-mode select (`dms`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dms: Option<VXToolModeSerde>,
}
