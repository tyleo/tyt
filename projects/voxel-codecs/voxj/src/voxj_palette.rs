use crate::VoxjValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A material palette: an attribute set with one row of values per cell.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VoxjPalette {
    /// Ordered attribute keys shared by every cell.
    pub attributes: Vec<String>,

    /// One row per cell, each value aligned to [`attributes`](Self::attributes);
    /// a cell is referenced by its row index.
    pub data: Vec<Vec<VoxjValue>>,
}
