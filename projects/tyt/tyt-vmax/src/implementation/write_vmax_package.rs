use crate::{ColorFormat, Result};
use std::{fs, path::Path};
use voxsmith::{VoxelMaxColorFormat, from_voxj_bytes, to_vmax_package};

/// Converts Voxel Json bytes into a `.vmax` package directory at `output`,
/// round-tripping through voxcore. `color_format` selects where each palette's
/// colors live.
pub(crate) fn write_vmax_package(
    voxj_bytes: &[u8],
    output: &Path,
    color_format: ColorFormat,
) -> Result<()> {
    let state = from_voxj_bytes(voxj_bytes)?;
    to_vmax_package(
        &state,
        voxel_max_color_format(color_format),
        |name, bytes| {
            let path = output.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            tyt_injection::write_file(&path, bytes)
        },
    )?;
    Ok(())
}

fn voxel_max_color_format(format: ColorFormat) -> VoxelMaxColorFormat {
    match format {
        ColorFormat::Png => VoxelMaxColorFormat::Png,
        ColorFormat::Plist => VoxelMaxColorFormat::Plist,
        ColorFormat::All => VoxelMaxColorFormat::All,
    }
}
