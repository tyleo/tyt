use clap::ValueEnum;

/// Which scene camera `to-vmax` writes into the rebuilt `.vmax`. Voxel Max opens a
/// document with nothing selected through its scene camera, which is every
/// document rebuilt from a non-Voxel-Max source, so this sets the view the file
/// opens to. Omitted, the input's `voxel-max` ext camera is kept when it has one,
/// else the empty default.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CameraView {
    /// Keep the camera from the input's `voxel-max` ext; errors without one.
    #[value(name = "ext")]
    Ext,
    /// Use the empty default camera.
    #[value(name = "empty")]
    Empty,
    /// Use the top-corner camera.
    #[value(name = "corner")]
    Corner,
}
