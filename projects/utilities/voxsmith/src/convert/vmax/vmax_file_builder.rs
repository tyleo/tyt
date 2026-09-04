use crate::{Result, SceneCameraSource, VMAX_DEPENDENCIES, VMaxColorFormat, VMaxFile, VMaxVoxMain};
use std::io::Result as IOResult;
use vmax_codec::to_vmax_package as write_vmax_package;
use vmax_voxcore::VmaxFileBuilder as RawVmaxFileBuilder;

/// Builds a Voxel Max document from a [`VMaxVoxMain`], the configurable
/// form of [`to_vmax_file`](crate::to_vmax_file) and
/// [`to_vmax_package`](crate::to_vmax_package). It defaults to PNG palette
/// colors and no camera override, reproducing the document those functions
/// write. The package terminal emits the built document one file at a time.
pub struct VmaxFileBuilder<'a>(RawVmaxFileBuilder<'a>);

impl<'a> VmaxFileBuilder<'a> {
    /// Starts a builder writing `state` into a Voxel Max document.
    pub fn new(state: &'a VMaxVoxMain) -> Self {
        Self(RawVmaxFileBuilder::new(state))
    }

    /// Sets where each palette's colors are stored.
    pub fn color_format(self, color_format: VMaxColorFormat) -> Self {
        Self(self.0.color_format(color_format))
    }

    /// Overrides the scene camera the document opens with. Unset, the camera
    /// is the ext's when present, else the empty default.
    pub fn scene_camera(self, scene_camera: SceneCameraSource) -> Self {
        Self(self.0.scene_camera(scene_camera))
    }

    /// Builds the lossless model.
    pub fn build(self) -> Result<VMaxFile> {
        Ok(self.0.build()?)
    }

    /// Builds the model and emits it as a package's files.
    ///
    /// # Arguments
    /// * `write` - receives each file's package-relative name and bytes and
    ///   performs the actual write, creating any subdirectory a `QuickLook/`
    ///   name implies.
    pub fn to_vmax_package<W>(self, mut write: W) -> Result<()>
    where
        W: FnMut(&str, &[u8]) -> IOResult<()>,
    {
        let file = self.build()?;
        write_vmax_package(&VMAX_DEPENDENCIES, &file, |name, bytes| {
            write(name, bytes).map_err(Into::into)
        })?;
        Ok(())
    }
}
