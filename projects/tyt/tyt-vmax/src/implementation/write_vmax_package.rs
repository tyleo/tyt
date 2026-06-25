use crate::{ColorFormat, Result};
use std::{fs, path::Path};
use vmax_codec::to_vmax_file;
use voxsmith::{VoxelMaxColorFormat, vmax_file_from_vox_state, vox_state_from_voxj_bytes};

/// Reconstructs a `.vmax` package directory at `output` from `.voxj` / `.voxjz`
/// bytes, round-tripping through voxcore: voxsmith loads the document into a
/// [`VoxState`](voxcore::VoxState) and back out to the lossless Voxel Max model,
/// which is then written one file per entry.
///
/// `color_format` selects where each palette's colors live: a 256x1
/// `palette*.png` image ([`ColorFormat::Png`]), the material
/// `palette*.settings.vmaxpsb` `colors` table ([`ColorFormat::Plist`]), or both
/// ([`ColorFormat::All`]). The `pal` references are written in every case.
pub(crate) fn write_vmax_package(
    voxj_bytes: &[u8],
    output: &Path,
    color_format: ColorFormat,
) -> Result<()> {
    // Translate through voxcore: voxj bytes -> VoxState -> vmax model.
    let state = vox_state_from_voxj_bytes(voxj_bytes)?;
    let serde = vmax_file_from_vox_state(&state, voxel_max_color_format(color_format))?;

    // Write each entry of the package into the output directory.
    to_vmax_file(&serde, |name, bytes| {
        let path = output.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(tyt_injection::write_file(&path, bytes)?)
    })?;
    Ok(())
}

/// Maps the CLI [`ColorFormat`] to voxsmith's [`VoxelMaxColorFormat`].
fn voxel_max_color_format(format: ColorFormat) -> VoxelMaxColorFormat {
    match format {
        ColorFormat::Png => VoxelMaxColorFormat::Png,
        ColorFormat::Plist => VoxelMaxColorFormat::Plist,
        ColorFormat::All => VoxelMaxColorFormat::All,
    }
}
