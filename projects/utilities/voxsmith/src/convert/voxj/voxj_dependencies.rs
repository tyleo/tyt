use voxj::DependenciesImpl as VoxjDependenciesImpl;

/// The voxj dependencies every Voxel Json wrapper here binds: voxj's own
/// base64 transcoding and deflate-size block cost.
pub(crate) const VOXJ_DEPENDENCIES: VoxjDependenciesImpl = VoxjDependenciesImpl;
