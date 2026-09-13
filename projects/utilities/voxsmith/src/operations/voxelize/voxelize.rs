use crate::{
    Result,
    dependencies::voxelize::DecodeImage,
    operations::voxelize::{VoxelizeOptions, mesh_input_from_mesh_main, voxelize_input},
};
use meshdoc::{MeshExt, MeshMain};
use voxcore::VoxMain;

/// Voxelizes the mesh document `main` under `options` into a [`VoxMain`] of
/// one object placed by one root node, its palette reduced when
/// `options.reduction` is set and left canonical: colors in material order,
/// ids compacted. Every object the hierarchy places rasterizes in world
/// space, its node transforms applied. `dependencies` decodes the
/// document's images for per-texel sampling. `fallback_name` names the
/// object when neither `options.name` nor the document does.
pub fn voxelize<D: DecodeImage, T: MeshExt>(
    dependencies: &D,
    main: &MeshMain<T>,
    fallback_name: &str,
    options: &VoxelizeOptions,
) -> Result<VoxMain> {
    let input = mesh_input_from_mesh_main(dependencies, main)?;

    voxelize_input(&input, fallback_name, options)
}
