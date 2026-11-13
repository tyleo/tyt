use crate::utilities::{FillMode, GridResolution};

/// The options [`tiled_vox_main`](crate::operations::sdf_doc::tiled_vox_main())
/// samples under.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfTileOptions {
    /// How the voxel size is chosen.
    pub resolution: GridResolution,

    /// How much of each part a tile keeps.
    pub fill_mode: FillMode,

    /// The cells along each edge of a tile.
    pub tile: u32,
}
