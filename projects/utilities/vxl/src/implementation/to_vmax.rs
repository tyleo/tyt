use crate::{
    Format, Result,
    commands::{CameraView, ColorFormat},
    implementation,
};
use std::{fs, path::Path};
use voxsmith::{SceneCameraSource, VMaxColorFormat, VMaxSceneCamera, VmaxFileBuilder};

/// The top-corner scene camera `--camera corner` writes: the empty camera's
/// framing at the origin, rotated to a three-quarter view looking down at the
/// model from a corner. App-specific to this CLI, so it lives here rather than
/// in voxsmith.
const TOP_CORNER_CAMERA: VMaxSceneCamera = VMaxSceneCamera {
    da: 0.0,
    ha: 0.19591325521469116,
    lda: 0.0,
    lha: 1.820913314819336,
    lwa: 0.5,
    o: [0.0, 0.0, 0.0],
    px: 0.0,
    py: 0.0,
    wa: 0.25,
    z: 512.0,
};

/// Converts the voxel file at `input` into a Voxel Max `.vmax` package
/// directory at `output`, round-tripping through voxcore. `color_format`
/// selects where each palette's colors live and `camera` the scene camera the
/// document opens with.
pub fn to_vmax(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    color_format: ColorFormat,
    camera: Option<CameraView>,
) -> Result<()> {
    let state = implementation::load_state_vmax(input, from)?;
    let mut builder = VmaxFileBuilder::new(&state).color_format(vmax_color_format(color_format));
    // Omitted, the path keeps its own camera: the ext's when present, else the
    // empty default.
    if let Some(camera) = camera {
        builder = builder.scene_camera(match camera {
            CameraView::Ext => SceneCameraSource::Ext,
            CameraView::Empty => SceneCameraSource::Empty,
            CameraView::Corner => SceneCameraSource::Camera(TOP_CORNER_CAMERA),
        });
    }
    builder.to_vmax_package(|name, bytes| {
        let path = output.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)
    })?;
    Ok(())
}

/// Maps the CLI [`ColorFormat`] to voxsmith's [`VMaxColorFormat`].
fn vmax_color_format(format: ColorFormat) -> VMaxColorFormat {
    match format {
        ColorFormat::Png => VMaxColorFormat::Png,
        ColorFormat::Plist => VMaxColorFormat::Plist,
        ColorFormat::All => VMaxColorFormat::All,
    }
}
