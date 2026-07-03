use crate::{
    AttributeSelector, CameraView, ColorFormat, EditState, FillMode, Format, GridResolution,
    HierarchyViews, MaterialMode, MeshFormat, PaletteListFields, PaletteListLayout,
    PaletteReduction, PaletteShowLayout, PatternView, ReportLayout, Result, SelectIndex,
    VoxjEncoding, VoxjFormat, Width,
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

    /// Voxelizes the mesh at `input` into a Voxel Json document at `output`,
    /// reading the mesh extent to size the grid, then rasterizing into it.
    ///
    /// # Arguments
    /// * `input` - the glTF or GLB mesh to read.
    /// * `from` - source mesh format, inferred from `input`'s extension when
    ///   `None`.
    /// * `output` - the `.voxj` or `.voxjz` document to write.
    /// * `resolution` - how the grid is sized: a voxel count along the longest
    ///   axis, or a real-world voxel size (recorded as the placing node's scale).
    /// * `fill_mode` - a solid body (flood-filled) or a hollow surface shell.
    /// * `material_mode` - where each voxel's color and material come from.
    /// * `fill_color` - the color of voxels a mode cannot sample, or `None` for
    ///   the `none` default.
    /// * `name` - object-name override; `None` uses the mesh's name, else the
    ///   input stem.
    /// * `reduction` - the palette cell cap and its clustering controls.
    /// * `encoding` - the per-object block encodings.
    /// * `format` - the output container and printing form.
    #[allow(clippy::too_many_arguments)]
    fn voxelize(
        &self,
        input: &Path,
        from: Option<MeshFormat>,
        output: &Path,
        resolution: GridResolution,
        fill_mode: FillMode,
        material_mode: MaterialMode,
        fill_color: Option<[u8; 4]>,
        name: Option<&str>,
        reduction: PaletteReduction,
        encoding: VoxjEncoding,
        format: VoxjFormat,
    ) -> Result<()>;

    /// Reports what the voxel file at `input` contains: a document summary, its
    /// palettes, and its objects.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `layout` - how to lay out the report.
    fn info(&self, input: &Path, from: Option<Format>, layout: ReportLayout) -> Result<()>;

    /// Validates the Voxel Json document at `input` against the format spec and
    /// writes a per-check report to standard output, then fails when any check
    /// failed so the process exits non-zero. Voxel Json only; the on-disk
    /// encoding the checks inspect exists in no other format.
    ///
    /// # Arguments
    /// * `input` - the `.voxj` or `.voxjz` document to validate, recognized by
    ///   its leading bytes.
    /// * `layout` - how to lay out the report.
    fn validate(&self, input: &Path, layout: ReportLayout) -> Result<()>;

    /// Prints a one-line-per-palette overview of the voxel file at `input`: each
    /// palette's index, ordered attribute keys, cell count, and the objects that
    /// reference it.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `filters` - palette-index selectors; a palette lists when any matches,
    ///   and given none every palette lists.
    /// * `fields` - which fields to render beside the always-shown index.
    /// * `layout` - how to lay out the listing.
    fn palette_list(
        &self,
        input: &Path,
        from: Option<Format>,
        filters: &[SelectIndex],
        fields: PaletteListFields,
        layout: PaletteListLayout,
    ) -> Result<()>;

    /// Prints the value collections named by `selectors` from the palettes in
    /// the voxel file at `input`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `selectors` - the `--attribute` selectors, each naming one or more
    ///   value collections, in render order.
    /// * `layout` - how to arrange the collections, and the serialization to
    ///   emit.
    /// * `width` - the width the `row` layouts wrap to.
    fn palette_show(
        &self,
        input: &Path,
        from: Option<Format>,
        selectors: &[AttributeSelector],
        layout: PaletteShowLayout,
        width: Width,
    ) -> Result<()>;

    /// Prints the scene graph of the voxel file at `input` as a tree, marking
    /// instanced nodes and listing nodes that are neither a root nor a child
    /// and objects no node places.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source format, inferred from `input`'s extension when `None`.
    /// * `pattern` - node-path globs and their collapse flags; when set, only
    ///   matched nodes and objects and their ancestors print.
    /// * `collapse_instances` - when true, expand a shared node's first
    ///   placement and print each later placement as a non-expanded stub.
    /// * `views` - the per-node and per-object subtrees to append: the node
    ///   transform, each object's edit- and runtime-grid origins, bounds, and
    ///   extents, and its referenced palettes.
    fn hierarchy_show(
        &self,
        input: &Path,
        from: Option<Format>,
        pattern: Option<PatternView>,
        collapse_instances: bool,
        views: HierarchyViews,
    ) -> Result<()>;

    /// Writes `contents` to standard output.
    ///
    /// # Arguments
    /// * `contents` - the bytes to write.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
