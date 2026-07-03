use crate::VoxjPaletteBinding;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A material palette: attribute-to-pool bindings and the distinct materials it
/// uses, stored column-major over those bindings. A voxel samples a material by
/// its index; resolving it reads down the columns into the bound pools.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjPalette {
    /// Ordered attribute-to-pool bindings. Binding order fixes the column order
    /// in [`materials`](Self::materials). No two bindings share an attribute.
    pub bindings: Vec<VoxjPaletteBinding>,

    /// Column-major materials: one inner array per binding, in binding order,
    /// each of the same length M, the material count. `materials[b][m]` is a
    /// value-index into the pool bound by binding `b`.
    pub materials: Vec<Vec<usize>>,
}
