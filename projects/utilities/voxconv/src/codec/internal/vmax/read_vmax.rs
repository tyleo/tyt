use crate::{Result, VoxDocumentFile, codec::into_ext_slot};
use vmax_voxcore::codec::{
    dependencies::{DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse},
    from_vmax_package,
};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Decodes a `.vmax` package's files into a state.
pub fn read_vmax<D, T>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<T>>
where
    D: DecompressLzfse + DecodeVMaxPlist + DecodePng + DecodeVMaxSceneJson,
    T: VoxExtSlot,
{
    let paths = files.iter().map(|file| file.path.clone()).collect();

    let state = from_vmax_package(
        dependencies,
        || Ok(paths),
        |path| {
            Ok(files
                .iter()
                .find(|file| file.path == path)
                .map(|file| file.bytes.clone()))
        },
    )?;

    into_ext_slot(state)
}
