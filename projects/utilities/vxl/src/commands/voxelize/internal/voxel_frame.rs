use crate::CliValue;
use voxsmith::operations::voxelize::VoxelFrame;

impl CliValue for VoxelFrame {
    const VARIANTS: &'static [Self] = &[VoxelFrame::World, VoxelFrame::Local];

    fn name(self) -> &'static str {
        match self {
            VoxelFrame::World => "world",
            VoxelFrame::Local => "local",
        }
    }

    fn help(self) -> &'static str {
        match self {
            VoxelFrame::World => {
                "Build each object's grid in world space with its node transforms applied, so \
                 every object's voxels align on one lattice"
            }
            VoxelFrame::Local => {
                "Build each object's grid in its own frame and keep the node's rotation and \
                 translation, so a repeated object voxelizes once"
            }
        }
    }
}
