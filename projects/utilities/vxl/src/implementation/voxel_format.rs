use crate::Format;
use voxsmith::VoxelFormat;

/// Maps the CLI [`Format`] to voxsmith's [`VoxelFormat`].
pub fn voxel_format(format: Format) -> VoxelFormat {
    match format {
        Format::Voxj => VoxelFormat::Voxj,
        Format::VMax => VoxelFormat::VMax,
        Format::MVox => VoxelFormat::MVox,
        Format::Goxl => VoxelFormat::Goxl,
        Format::Qbcl => VoxelFormat::Qbcl,
    }
}
