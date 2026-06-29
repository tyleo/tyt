use crate::{CameraView, ColorFormat, EditState, Format, Result, VoxjEncoding, VoxjFormat};
use std::path::Path;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Converts the voxel file at `input` into a Goxel `.gox` file at `output`.
    /// `from` names the source format, inferred from `input`'s extension when
    /// `None`.
    fn to_goxl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()>;

    /// Converts the voxel file at `input` into a MagicaVoxel `.vox` file at
    /// `output`. `from` names the source format, inferred from `input`'s
    /// extension when `None`.
    fn to_mvox(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()>;

    /// Converts the voxel file at `input` into a Qubicle `.qbcl` file at
    /// `output`. `from` names the source format, inferred from `input`'s
    /// extension when `None`.
    fn to_qbcl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()>;

    /// Converts the voxel file at `input` into a Voxel Max `.vmax` package
    /// directory at `output`. `from` names the source format, inferred from
    /// `input`'s extension when `None`. `color_format` selects where each
    /// object's colors are stored in the package. `camera` selects the scene
    /// camera the rebuilt document opens with.
    fn to_vmax(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        color_format: ColorFormat,
        camera: Option<CameraView>,
    ) -> Result<()>;

    /// Converts the voxel file at `input` into a Voxel Json document at
    /// `output`. `from` names the source format, inferred from `input`'s
    /// extension when `None`. `encoding` selects the per-object block encodings
    /// and `format` the output container and printing form. When `ext` is false,
    /// the user-defined `ext` extension block is omitted from the output.
    /// `edit_state` selects when each object's editor build volume is recorded.
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
