use crate::{CameraView, ColorFormat, EditState, Format, Result, VoxjEncoding, VoxjFormat};
use std::path::Path;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Converts the voxel file at `input` into a Goxel `.gox` file at `output`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `output` - the `.gox` file to write.
    fn to_goxl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()>;

    /// Converts the voxel file at `input` into a MagicaVoxel `.vox` file at
    /// `output`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `output` - the `.vox` file to write.
    fn to_mvox(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()>;

    /// Converts the voxel file at `input` into a Qubicle `.qbcl` file at
    /// `output`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `output` - the `.qbcl` file to write.
    fn to_qbcl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()>;

    /// Converts the voxel file at `input` into a Voxel Max `.vmax` package
    /// directory at `output`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `output` - the `.vmax` package directory to write.
    /// * `color_format` - where each object's colors are stored in the package.
    /// * `camera` - the scene camera the rebuilt document opens with.
    fn to_vmax(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        color_format: ColorFormat,
        camera: Option<CameraView>,
    ) -> Result<()>;

    /// Converts the voxel file at `input` into a Voxel Json document at
    /// `output`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `output` - the `.voxj` or `.voxjz` document to write.
    /// * `encoding` - the per-object block encodings.
    /// * `format` - the output container and printing form.
    /// * `ext` - when false, omits the user-defined `ext` extension block.
    /// * `edit_state` - when to record each object's editor build volume.
    #[allow(clippy::too_many_arguments)]
    fn to_voxj(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        encoding: VoxjEncoding,
        format: VoxjFormat,
        ext: bool,
        edit_state: EditState,
    ) -> Result<()>;
}
