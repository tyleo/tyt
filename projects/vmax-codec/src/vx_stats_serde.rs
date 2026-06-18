use serde::Deserialize;

/// Per-snapshot statistics (`s.st`). Only `min` is needed: its fourth element
/// is the Morton code of the snapshot's first `ds` slot.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct VXStatsSerde {
    #[serde(default)]
    pub min: Vec<i64>,
}
