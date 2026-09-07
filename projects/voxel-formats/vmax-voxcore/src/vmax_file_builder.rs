use crate::{Result, SceneCameraSource, VMaxColorFormat, ext::VMaxExt, write_vmax};
use vmax::VMaxFile;
use voxcore::VoxMain;

/// Builds a [`VMaxFile`] from a state, the configurable form of
/// [`to_vmax_file`](crate::to_vmax_file). It defaults to PNG colors and keeps
/// the scene camera the path produces, the same document that function writes.
/// [`new`](VmaxFileBuilder::new) starts from a bare state. The `ext` feature
/// adds `new_with_ext`, which starts from a state carrying its ext.
pub struct VmaxFileBuilder<'a, T> {
    state: &'a VoxMain<T>,
    vmax_ext: Option<&'a VMaxExt>,
    color_format: VMaxColorFormat,
    scene_camera: Option<SceneCameraSource>,
}

impl<'a> VmaxFileBuilder<'a, ()> {
    /// Starts a builder writing `state` as a document synthesized from its
    /// scene.
    pub fn new(state: &'a VoxMain<()>) -> Self {
        Self::with_ext(state, None)
    }
}

impl<'a, T> VmaxFileBuilder<'a, T> {
    pub(crate) fn with_ext(state: &'a VoxMain<T>, vmax_ext: Option<&'a VMaxExt>) -> Self {
        Self {
            state,
            vmax_ext,
            color_format: VMaxColorFormat::Png,
            scene_camera: None,
        }
    }

    /// Sets where each palette's colors are stored.
    pub fn color_format(mut self, color_format: VMaxColorFormat) -> Self {
        self.color_format = color_format;
        self
    }

    /// Overrides the scene camera the document opens with. Left unset, the
    /// path's own camera is kept.
    pub fn scene_camera(mut self, scene_camera: SceneCameraSource) -> Self {
        self.scene_camera = Some(scene_camera);
        self
    }

    /// Builds the [`VMaxFile`].
    pub fn build(self) -> Result<VMaxFile> {
        write_vmax(
            self.state,
            self.vmax_ext,
            self.color_format,
            self.scene_camera,
        )
    }
}
