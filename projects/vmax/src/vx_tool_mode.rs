use crate::VXMode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Voxel Max tool-mode entry (`tools.ct`, `tools.dms`, …): a dictionary keyed by
/// editor surface, each value configuring that surface's mode. The surface set is
/// fixed (`a`, `b`, `c`, `e`, `l`, `m`, `p`, `s`), so each is an optional field —
/// absent surfaces stay `None` and are skipped on serialize. A tool may carry more
/// than one surface at once (Voxel Max history records up to four).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXToolMode {
    /// Add surface (`a`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub a: Option<VXMode>,
    /// Brush surface (`b`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub b: Option<VXMode>,
    /// Color/cube surface (`c`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub c: Option<VXMode>,
    /// Erase/edge surface (`e`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub e: Option<VXMode>,
    /// Layer surface (`l`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub l: Option<VXMode>,
    /// Material surface (`m`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub m: Option<VXMode>,
    /// Paint surface (`p`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub p: Option<VXMode>,
    /// Select surface (`s`).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub s: Option<VXMode>,
}
