use crate::{Result, VoxDocumentFile, package_file};
use vmax_voxcore::codec::{
    dependencies::{DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse},
    from_vmax_package,
};
use voxcore::VoxMain;

/// Decodes a `.vmax` package's files into a bare state.
pub fn read_vmax<D>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>>
where
    D: DecompressLzfse + DecodeVMaxPlist + DecodePng + DecodeVMaxSceneJson,
{
    let paths = files.iter().map(|file| file.path.clone()).collect();

    Ok(from_vmax_package(
        dependencies,
        || Ok(paths),
        |path| Ok(package_file(files, path)),
    )?)
}
