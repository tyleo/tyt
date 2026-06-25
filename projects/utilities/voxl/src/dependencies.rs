use crate::{ColorFormat, Format, Result, VoxjEncoding, VoxjFormat};
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
    /// object's colors are stored in the package.
    fn to_vmax(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        color_format: ColorFormat,
    ) -> Result<()>;

    /// Converts the voxel file at `input` into a Voxel Json document at
    /// `output`. `from` names the source format, inferred from `input`'s
    /// extension when `None`. `encoding` selects the per-object block encodings
    /// and `format` the output container and printing form.
    fn to_voxj(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        encoding: VoxjEncoding,
        format: VoxjFormat,
    ) -> Result<()>;
}
