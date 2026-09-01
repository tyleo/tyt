use crate::Format;
use voxsmith::VoxelFormat;

/// Maps the CLI [`Format`] to voxsmith's [`VoxelFormat`].
pub fn voxel_format(format: Format) -> VoxelFormat {
    match format {
        Format::Voxj => VoxelFormat::VoxelJson,
        Format::VMax => VoxelFormat::VoxelMax,
        Format::MVox => VoxelFormat::MagicaVoxel,
        Format::Goxl => VoxelFormat::Goxel,
        Format::Qbcl => VoxelFormat::QubicleQbcl,
    }
}
