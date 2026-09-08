use crate::{Result, VoxDocumentFile, retype_ext, vmax::VMaxWriteOptions};
use vmax_voxcore::codec::{
    dependencies::{CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson},
    to_vmax_package_with_ext,
};
use voxcore::{VoxMain, ext::VoxExt};

/// Encodes a state with a boxed ext as a `.vmax` package's files.
pub fn write_vmax_with_ext<D>(
    dependencies: &D,
    state: VoxMain<Box<dyn VoxExt>>,
    options: &VMaxWriteOptions,
) -> Result<Vec<VoxDocumentFile>>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
{
    let state = retype_ext(state)?;

    let mut files = Vec::new();

    to_vmax_package_with_ext(dependencies, &state, options, |path, bytes| {
        files.push(VoxDocumentFile::new(path, bytes.to_vec()));

        Ok(())
    })?;

    Ok(files)
}
