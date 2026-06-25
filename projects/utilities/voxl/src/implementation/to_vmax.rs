use crate::{ColorFormat, Format, Result};
use std::{fs, path::Path};
use vmax_codec::to_vmax_package;
use voxsmith::{VoxelMaxColorFormat, to_vmax_file};

/// Converts the voxel file at `input` into a Voxel Max `.vmax` package directory
/// at `output`, round-tripping through voxcore: the input is loaded into a
/// [`VoxState`](voxcore::VoxState), written back out to the lossless Voxel Max
/// model, then emitted one file per package entry. `color_format` selects where
/// each palette's colors live.
pub(crate) fn to_vmax(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    color_format: ColorFormat,
) -> Result<()> {
    let state = super::load_state::load_state(input, from)?;
    let serde = to_vmax_file(&state, voxel_max_color_format(color_format))?;
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
