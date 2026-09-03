use crate::{Result, VoxelMaxVoxMain, from_vmax_file};
use vmax_codec::{
    DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse, Result as CodecResult,
    from_vmax_package as read_vmax_package,
};

/// Loads a `.vmax` package into a [`VoxelMaxVoxMain`] through `dependencies`,
/// the package form of [`from_vmax_file`].
///
/// # Arguments
/// * `list` - returns the package-relative path of every file, so `QuickLook/`
///   entries keep their subdirectory prefix.
/// * `resolve` - returns a file's bytes by that path, or `Ok(None)` if it has
///   since vanished.
pub fn from_vmax_package<D, L, R>(dependencies: &D, list: L, resolve: R) -> Result<VoxelMaxVoxMain>
where
    D: DecompressLzfse + DecodeVMaxPlist + DecodePng + DecodeVMaxSceneJson,
    L: FnOnce() -> CodecResult<Vec<String>>,
    R: FnMut(&str) -> CodecResult<Option<Vec<u8>>>,
{
    let file = read_vmax_package(dependencies, list, resolve)?;
    from_vmax_file(&file)
}
