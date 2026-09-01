use crate::{ColorFormat, Result};
use std::{fs, path::Path};
use vmax_codec::to_vmax_package;
use voxj_voxcore::codec::from_voxj_bytes;
use voxsmith::{VoxelMaxColorFormat, to_vmax_file};

/// Reconstructs a `.vmax` package directory at `output` from `.voxj` / `.voxjz`
/// bytes, round-tripping through voxcore: voxj-voxcore loads the document into
/// a [`VoxMain`](voxcore::VoxMain), voxsmith writes it back out to the lossless
/// Voxel Max model, and the model lands one file per entry.
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
    // Translate through voxcore: voxj bytes -> VoxMain -> vmax model.
    let state = from_voxj_bytes(voxj_bytes)?;
    let serde = to_vmax_file(&state, voxel_max_color_format(color_format))?;

    // Write each entry of the package into the output directory.
    to_vmax_package(&serde, |name, bytes| {
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
