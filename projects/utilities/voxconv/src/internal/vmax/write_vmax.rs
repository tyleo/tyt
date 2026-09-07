use crate::{Result, VoxDocumentFile, retype_ext, vmax::VMaxWriteOptions};
use vmax_voxcore::{
    VmaxFileBuilder,
    codec::{
        VmaxFileBuilderCodec,
        dependencies::{CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson},
    },
};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Encodes a state as a `.vmax` package's files.
pub fn write_vmax<D, T>(
    dependencies: &D,
    state: VoxMain<T>,
    options: &VMaxWriteOptions,
) -> Result<Vec<VoxDocumentFile>>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
    T: VoxExtBlockCodec,
{
    let state = retype_ext(state)?;

    let mut builder = VmaxFileBuilder::new(&state).color_format(options.color_format);

    if let Some(scene_camera) = options.scene_camera {
        builder = builder.scene_camera(scene_camera);
    }

    let mut files = Vec::new();

    builder.to_vmax_package(dependencies, |path, bytes| {
        files.push(VoxDocumentFile::new(path, bytes.to_vec()));

        Ok(())
    })?;

    Ok(files)
}
