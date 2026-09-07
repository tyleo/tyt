use crate::{
    Result,
    ext::{VMaxVoxMain, from_vmax_file_with_ext},
};
use vmax_codec::{
    DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse, Result as CodecResult,
    from_vmax_package as read_vmax_package,
};

/// Loads a `.vmax` package through `dependencies` into a [`VMaxVoxMain`]
/// carrying its ext, the package form of [`from_vmax_file_with_ext`]. `list`
/// and `resolve` work as on
/// [`from_vmax_package`](crate::codec::from_vmax_package).
pub fn from_vmax_package_with_ext<D, L, R>(
    dependencies: &D,
    list: L,
    resolve: R,
) -> Result<VMaxVoxMain>
where
    D: DecompressLzfse + DecodeVMaxPlist + DecodePng + DecodeVMaxSceneJson,
    L: FnOnce() -> CodecResult<Vec<String>>,
    R: FnMut(&str) -> CodecResult<Option<Vec<u8>>>,
{
    let file = read_vmax_package(dependencies, list, resolve)?;

    from_vmax_file_with_ext(&file)
}
