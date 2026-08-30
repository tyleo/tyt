use crate::VMaxMode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Voxel Max tool-mode entry (`tools.ct`, `tools.dms`, ...): a dictionary
/// keyed by editor surface. A tool may carry more than one surface at once
/// (history records up to four).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxToolMode {
    /// Add surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub a: Option<VMaxMode>,

    /// Brush surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub b: Option<VMaxMode>,

    /// Color/cube surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub c: Option<VMaxMode>,

    /// Erase/edge surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub e: Option<VMaxMode>,

    /// Layer surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub l: Option<VMaxMode>,

    /// Material surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub m: Option<VMaxMode>,

    /// Paint surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub p: Option<VMaxMode>,

    /// Select surface.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub s: Option<VMaxMode>,
}
