use crate::{Error, PositiveF64, Result, commands::ResolutionEntry};
use voxsmith::utilities::GridResolution;

/// The grid resolution a voxelize profile's `resolution` or `voxelSize` sets.
/// Errors when the profile sets both.
pub fn profile_grid_resolution(
    resolution: Option<ResolutionEntry>,
    voxel_size: Option<PositiveF64>,
) -> Result<Option<GridResolution>> {
    match (resolution, voxel_size) {
        (Some(_), Some(_)) => Err(Error::usage(
            "a profile sets `resolution` or `voxelSize`, not both",
        )),

        (Some(resolution), None) => Ok(Some(resolution.into())),

        (None, Some(size)) => Ok(Some(GridResolution::VoxelSize(size.0))),

        (None, None) => Ok(None),
    }
}
