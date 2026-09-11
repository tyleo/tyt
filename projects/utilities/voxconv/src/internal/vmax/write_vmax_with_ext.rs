use crate::{
    Result, VoxDocumentFile, ext::VoxconvVoxMain, find_ext, vmax::VMaxWriteOptions, write_vmax,
};
use vmax_voxcore::{
    codec::{
        dependencies::{CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson},
        to_vmax_package_with_ext,
    },
    ext::VMaxExt,
};

/// Encodes a boxed state as a `.vmax` package's files: exactly through the
/// Voxel Max ext the box holds or encodes, else synthesized like the bare
/// pair.
pub fn write_vmax_with_ext<D>(
    dependencies: &D,
    state: VoxconvVoxMain,
    options: &VMaxWriteOptions,
) -> Result<Vec<VoxDocumentFile>>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
{
    let Some(ext) = find_ext::<VMaxExt>(state.ext().as_ref())? else {
        return write_vmax(dependencies, &state.take_ext().0, options);
    };
    let state = state.take_ext().0.put_ext(ext);
    let mut files = Vec::new();
    to_vmax_package_with_ext(dependencies, &state, options, |path, bytes| {
        files.push(VoxDocumentFile::new(path, bytes.to_vec()));
        Ok(())
    })?;
    Ok(files)
}
