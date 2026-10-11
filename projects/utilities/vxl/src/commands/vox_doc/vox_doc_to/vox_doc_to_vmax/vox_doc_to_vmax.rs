use crate::{
    Dependencies, ObjectSelection, Result, VoxelInput, cli_value_parser,
    commands::{CameraView, convert},
};
use clap::Parser;
use std::path::PathBuf;
use voxconv::{
    WriteFormat,
    vmax::{SceneCameraSource, VMaxColorFormat, VMaxObjectSize, VMaxSceneCamera, VMaxWriteOptions},
};

/// The top-corner scene camera `--camera corner` writes: the empty camera's
/// framing at the origin, rotated to a three-quarter view looking down at the
/// model from a corner. Specific to this CLI.
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
    aq: None,
    op: None,
    zf: None,
};

/// Converts a voxel file to the Voxel Max format.
#[derive(Clone, Debug, Parser)]
#[command(name = "vmax")]
pub struct VoxDocToVmax {
    #[command(flatten)]
    input: VoxelInput,

    /// The output `.vmax` package directory to write. Defaults to the input
    /// path with a `.vmax` extension.
    #[arg(value_name = "output")]
    output: Option<PathBuf>,

    /// Where to store object colors in the package.
    #[arg(
        value_name = "color-format",
        long,
        default_value = "png",
        value_parser = cli_value_parser::<VMaxColorFormat>()
    )]
    color_format: VMaxColorFormat,

    /// The editable cube width for each voxel object. A fixed size compacts
    /// empty canvas margins; voxels and scene pivots keep their world positions.
    #[arg(value_name = "object-size", long, default_value = "auto", value_parser = cli_value_parser::<VMaxObjectSize>())]
    object_size: VMaxObjectSize,

    /// Which scene camera the rebuilt document opens with. When omitted, the
    /// input's `vmax` ext camera is kept when present, else the empty default.
    #[arg(value_name = "camera", long)]
    camera: Option<CameraView>,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl VoxDocToVmax {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let to = WriteFormat::VMax(VMaxWriteOptions {
            color_format: self.color_format,
            scene_camera: resolve_scene_camera(self.camera),
            object_size: self.object_size,
        });

        let output = self.input.output_path(self.output, to.extension());

        convert(
            &dependencies,
            &self.input.path,
            from,
            &output,
            &to,
            &self.selection,
        )
    }
}

/// The scene camera the writer takes for a `--camera` choice. Omitted, the
/// writer keeps the ext's camera.
fn resolve_scene_camera(camera: Option<CameraView>) -> SceneCameraSource {
    match camera {
        None | Some(CameraView::Ext) => SceneCameraSource::Ext,
        Some(CameraView::Empty) => SceneCameraSource::Empty,
        Some(CameraView::Corner) => SceneCameraSource::Camera(TOP_CORNER_CAMERA),
    }
}
