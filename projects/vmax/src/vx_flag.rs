use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single-value Voxel Max tool flag (`{x: …}`) — used by `tools.mr`, `tools.st`
/// (boolean) and `tools.stf` (integer). The payload is kept as a generic
/// [`Value`] so the one struct round-trips either kind without coercion.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VXFlag {
    /// The flag value (`x`), a boolean or integer depending on the flag.
    pub x: Value,
}
