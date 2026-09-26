use crate::CliValue;
use voxsmith::operations::voxelize::VoxelScale;

impl CliValue for VoxelScale {
    const VARIANTS: &'static [Self] = &[VoxelScale::Bake, VoxelScale::Keep];

    fn name(self) -> &'static str {
        match self {
            VoxelScale::Bake => "bake",
            VoxelScale::Keep => "keep",
        }
    }

    fn help(self) -> &'static str {
        match self {
            VoxelScale::Bake => {
                "Apply node scale to the geometry, so every object's cubes are the voxel size"
            }
            VoxelScale::Keep => {
                "Keep node scale on the node, so a scaled object's cubes scale with it"
            }
        }
    }
}
