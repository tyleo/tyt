use crate::{ColorFormat, Result};
use std::path::Path;
use voxconv::{
    DependenciesImpl, ReadFormat, VoxDocumentFile, WriteFormat,
    ext::{read_with_ext, save_with_ext},
    vmax::{VMaxColorFormat, VMaxWriteOptions},
};

/// Converts Voxel Json bytes into a `.vmax` package directory at `output`,
/// round-tripping through voxcore. `color_format` selects where each palette's
/// colors live.
pub(crate) fn write_vmax_package(
    voxj_bytes: &[u8],
    output: &Path,
    color_format: ColorFormat,
) -> Result<()> {
    let files = [VoxDocumentFile::single(voxj_bytes.to_vec())];

    // A `vmax` entry in the document's `ext` block decodes back into the
    // package.
    let state = read_with_ext(&DependenciesImpl, ReadFormat::Voxj, &files)?;

    let options = VMaxWriteOptions {
        color_format: vmax_color_format(color_format),
        scene_camera: None,
    };

    save_with_ext(
        &DependenciesImpl,
        &WriteFormat::VMax(options),
        state,
        output,
    )?;

    Ok(())
}

fn vmax_color_format(format: ColorFormat) -> VMaxColorFormat {
    match format {
        ColorFormat::Png => VMaxColorFormat::Png,
        ColorFormat::Plist => VMaxColorFormat::Plist,
        ColorFormat::All => VMaxColorFormat::All,
    }
}
