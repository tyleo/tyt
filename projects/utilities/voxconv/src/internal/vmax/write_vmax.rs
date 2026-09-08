use crate::{Result, VoxDocumentFile, vmax::VMaxWriteOptions};
use vmax_voxcore::codec::{
    dependencies::{CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson},
    to_vmax_package,
};
use voxcore::VoxMain;

/// Encodes a bare state as a `.vmax` package's files.
pub fn write_vmax<D>(
    dependencies: &D,
    state: &VoxMain<()>,
    options: &VMaxWriteOptions,
) -> Result<Vec<VoxDocumentFile>>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
{
    let mut files = Vec::new();

    to_vmax_package(dependencies, state, options, |path, bytes| {
        files.push(VoxDocumentFile::new(path, bytes.to_vec()));

        Ok(())
    })?;

    Ok(files)
}
