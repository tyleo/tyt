use crate::VXBrushColor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One slot in a Voxel Max brush palette (`brush.brushes[]`): a dictionary keyed
/// by brush-slot type (`c`, `ch`, `e`, `eh`, `bb`, `db`, `pr`, `py`, `et`, …)
/// whose value is the slot's color payload. Modeled as a map so any slot type
/// round-trips without enumerating Voxel Max's key names.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct VXBrushEntry(pub BTreeMap<String, VXBrushColor>);
