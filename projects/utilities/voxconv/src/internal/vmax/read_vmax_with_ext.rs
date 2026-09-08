use crate::{Result, VoxDocumentFile, box_ext, package_file};
use vmax_voxcore::codec::{
    dependencies::{DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse},
    from_vmax_package_with_ext,
};
use voxcore::{VoxMain, ext::VoxExt};

/// Decodes a `.vmax` package's files into a state carrying its ext.
pub fn read_vmax_with_ext<D>(
    dependencies: &D,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<Box<dyn VoxExt>>>
where
    D: DecompressLzfse + DecodeVMaxPlist + DecodePng + DecodeVMaxSceneJson,
{
    let paths = files.iter().map(|file| file.path.clone()).collect();

    Ok(box_ext(from_vmax_package_with_ext(
        dependencies,
        || Ok(paths),
        |path| Ok(package_file(files, path)),
    )?))
}
