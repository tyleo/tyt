use crate::{
    AttributeType, CameraView, ColorComponent, ColorFormat, EditState, Format, InfoLayout,
    PaletteShowFormat, Result, VoxjEncoding, VoxjFormat,
};
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

    /// Reports what the voxel file at `input` contains: a document summary, its
    /// palettes, and its objects.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `layout` - how to lay out the report.
    fn info(&self, input: &Path, from: Option<Format>, layout: InfoLayout) -> Result<()>;

    /// Prints the selected attribute of one palette in the voxel file at
    /// `input`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `index` - which palette to show, by index into the document's palettes.
    /// * `attribute` - the attribute key to show.
    /// * `component` - one color component to extract, or `None` for the whole
    ///   value.
    /// * `r#type` - how to interpret the values, inferred from the stored value
    ///   when `None`.
    /// * `format` - how to render each value.
    /// * `json` - emit the selected attribute as JSON instead.
    #[allow(clippy::too_many_arguments)]
    fn palette_show(
        &self,
        input: &Path,
        from: Option<Format>,
        index: usize,
        attribute: &str,
        component: Option<ColorComponent>,
        r#type: Option<AttributeType>,
        format: PaletteShowFormat,
        json: bool,
    ) -> Result<()>;

    /// Matches `pattern` against each candidate hierarchy path, returning one
    /// boolean per candidate in order. The pattern is the project's standard
    /// glob, built with path separators literal.
    ///
    /// # Arguments
    /// * `pattern` - the glob to match, already `**/`-normalized by the caller.
    /// * `candidates` - the hierarchy paths to test, in order.
    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>>;

    /// Writes `contents` to standard output.
    ///
    /// # Arguments
    /// * `contents` - the bytes to write.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
