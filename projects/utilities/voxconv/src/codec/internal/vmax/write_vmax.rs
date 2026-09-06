use crate::{Result, VoxDocumentFile, codec::into_ext_slot, vmax::VMaxWriteOptions};
use vmax_voxcore::{
    VmaxFileBuilder,
    codec::{
        VmaxFileBuilderCodec,
        dependencies::{CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson},
    },
};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Encodes a state as a `.vmax` package's files.
pub fn write_vmax<D, T>(
    dependencies: &D,
    state: VoxMain<T>,
    options: &VMaxWriteOptions,
) -> Result<Vec<VoxDocumentFile>>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
    T: VoxExtSlot,
{
    let state = into_ext_slot(state)?;

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
