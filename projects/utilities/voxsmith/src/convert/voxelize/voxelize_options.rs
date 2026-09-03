use crate::{
    FillMode, GridResolution, MaterialMode, OutOfRangeProperty, PaletteReduction, SurfaceMode,
};

/// The options [`from_mesh`](crate::from_mesh) voxelizes under.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelizeOptions {
    /// How the grid is sized from the mesh extent.
    pub resolution: GridResolution,

    /// How the surface decides which cells it occupies.
    pub surface_mode: SurfaceMode,

    /// Whether the interior is filled or the result is a hollow shell.
    pub fill_mode: FillMode,

    /// Where each voxel's color and material come from.
    pub material_mode: MaterialMode,

    /// The straight sRGB color of voxels a mode cannot sample, or `None` for
    /// the default: white under [`MaterialMode::Flat`], the nearest surface
    /// color for a solid interior.
    pub fill_color: Option<[u8; 4]>,

    /// The object name, overriding the mesh's name.
    pub name: Option<String>,

    /// Whether a material value outside its property's glTF range errors or
    /// clamps.
    pub out_of_range_property: OutOfRangeProperty,

    /// The palette reduction to apply to the generated palette, or `None` to
    /// keep every sampled material.
    pub reduction: Option<PaletteReduction>,
}
