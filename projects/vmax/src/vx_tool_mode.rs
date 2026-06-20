use crate::VXMode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A Voxel Max tool-mode entry (`tools.ct`, `tools.dms`, …): a dictionary keyed by
/// editor surface (`c`, `e`, `s`, `m`, `a`, `b`, `l`, `p`, …) whose value
/// configures that surface's mode. Modeled as a map so any surface round-trips
/// without enumerating Voxel Max's cryptic, version-varying key names.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct VXToolMode(pub BTreeMap<String, VXMode>);
