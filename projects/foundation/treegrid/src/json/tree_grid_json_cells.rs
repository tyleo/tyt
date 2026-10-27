use crate::TreeGridCells;
use serde_json::Value;

/// The JSON side of a cell policy: each value's native JSON form for
/// the JSON layouts.
///
/// A separate trait so the JSON renders are callable only on a grid
/// whose policy provides JSON forms; a text-only policy never meets
/// the bound. Exists behind the crate's non-default `json` feature.
pub trait TreeGridJsonCells: TreeGridCells {
    /// The value's native JSON form.
    fn json(&self, value: &Self::Value) -> Value;
}
