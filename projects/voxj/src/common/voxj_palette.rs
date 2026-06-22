use crate::VoxjValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A palette: an ordered attribute set plus one row of values per cell.
///
/// Cell `c`'s value for [`attributes`](Self::attributes)`[i]` is
/// [`data`](Self::data)`[c][i]`; every row has exactly `attributes.len()`
/// values, and a cell is referenced by its row index.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VoxjPalette {
    pub attributes: Vec<String>,

    pub data: Vec<Vec<VoxjValue>>,
}
