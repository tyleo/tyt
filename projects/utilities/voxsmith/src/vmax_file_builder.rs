use crate::{Result, SceneCameraSource, VoxelMaxColorFormat, write_vmax};
use vmax::VMaxFile;
use voxcore::VoxState;

/// Builds a [`VMaxFile`] from a [`VoxState`], the configurable form of
/// [`to_vmax_file`](crate::to_vmax_file). It defaults to PNG colors and keeps the
/// scene camera the path produces, the same document that function writes.
pub struct VmaxFileBuilder<'a> {
    state: &'a VoxState,
    color_format: VoxelMaxColorFormat,
    scene_camera: Option<SceneCameraSource>,
}

impl<'a> VmaxFileBuilder<'a> {
    /// Starts a builder writing `state` back to a Voxel Max document.
    pub fn new(state: &'a VoxState) -> Self {
        Self {
            state,
            color_format: VoxelMaxColorFormat::Png,
            scene_camera: None,
        }
    }

    /// Sets where each palette's colors are stored.
    pub fn color_format(mut self, color_format: VoxelMaxColorFormat) -> Self {
        self.color_format = color_format;
        self
    }

    /// Overrides the scene camera the document opens with. Left unset, the path's
    /// own camera is kept.
    pub fn scene_camera(mut self, scene_camera: SceneCameraSource) -> Self {
        self.scene_camera = Some(scene_camera);
        self
    }

    /// Builds the [`VMaxFile`].
    pub fn build(self) -> Result<VMaxFile> {
        write_vmax(self.state, self.color_format, self.scene_camera)
    }
}
