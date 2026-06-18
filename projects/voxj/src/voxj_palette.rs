use crate::AttrValue;

/// A palette: an ordered attribute set plus one row of values per cell.
///
/// Cell `c`'s value for `attributes[i]` is `data[c][i]`; every row has exactly
/// `attributes.len()` values, and a cell is referenced by its row index.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjPalette {
    pub attributes: Vec<String>,
    pub data: Vec<Vec<AttrValue>>,
}
