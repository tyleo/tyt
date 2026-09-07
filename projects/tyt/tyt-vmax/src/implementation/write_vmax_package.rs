use crate::{ColorFormat, Result};
use std::path::Path;
use voxconv::{
    DependenciesImpl, ReadFormat, VoxDocumentFile, WriteFormat, read, save,
    vmax::{VMaxColorFormat, VMaxWriteOptions},
};
use voxcore::{VoxMain, VoxMap};

/// Converts Voxel Json bytes into a `.vmax` package directory at `output`,
/// round-tripping through voxcore. `color_format` selects where each palette's
/// colors live.
pub(crate) fn write_vmax_package(
    voxj_bytes: &[u8],
    output: &Path,
    color_format: ColorFormat,
) -> Result<()> {
    let files = [VoxDocumentFile::single(voxj_bytes.to_vec())];

    // A `vmax` block in the document's `ext` decodes back into the package.
    let state: VoxMain<Option<VoxMap>> = read(&DependenciesImpl, ReadFormat::Voxj, &files)?;

    let options = VMaxWriteOptions {
        color_format: vmax_color_format(color_format),
        scene_camera: None,
    };

    save(
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
