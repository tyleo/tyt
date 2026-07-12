use crate::{
    AttributeSelector, CameraView, ColorFormat, EditState, FillMode, Format, GridResolution,
    HierarchyViews, MaterialMode, MeshFormat, MeshMethod, MeshTextureMap, PaletteListFields,
    PaletteListLayout, PaletteReduction, PaletteShowLayout, PatternView, ReportLayout,
    ResourceStorage, Result, SelectIndex, TextureShape, VoxjColorFormat, VoxjEncoding, VoxjFormat,
    Width,
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
    /// * `color_format` - the on-wire encoding for sRGB color pools.
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
        color_format: VoxjColorFormat,
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
    /// * `fill_color` - the color of voxels a mode cannot sample, or `None` when
    ///   the flag is omitted.
    /// * `name` - object-name override; `None` uses the mesh's name, else the
    ///   input stem.
    /// * `reduction` - the palette material cap and its clustering controls.
    /// * `encoding` - the per-object block encodings.
    /// * `format` - the output container and printing form.
    /// * `color_format` - the on-wire encoding for sRGB color pools.
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
        color_format: VoxjColorFormat,
    ) -> Result<()>;

    /// Resolves the object selectors against the voxel file at `input` to the
    /// indices of the matching objects, in document order and deduplicated. The
    /// caller applies its own policy to the result. With neither selector every
    /// object matches.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source voxel format, inferred from `input`'s extension when
    ///   `None`.
    /// * `select` - hierarchy-path globs, gitignore-style, matched as
    ///   `hierarchy show` matches node paths; a matched node contributes its
    ///   subtree, a matched object contributes itself.
    /// * `select_index` - object-index selectors, an integer or an `a-b` range.
    fn resolve_objects(
        &self,
        input: &Path,
        from: Option<Format>,
        select: &[String],
        select_index: &[SelectIndex],
    ) -> Result<Vec<usize>>;

    /// Meshes the object at index `object` of the voxel file at `input` into a
    /// glTF or GLB mesh at `output`, with no hierarchy-node transform. With no
    /// `maps` it is pure geometry; otherwise it bakes the palette materials into
    /// textures the mesh's UVs sample, writing any loose images beside `output`.
    ///
    /// # Arguments
    /// * `input` - the voxel file to read, in any supported format.
    /// * `from` - source voxel format, inferred from `input`'s extension when
    ///   `None`.
    /// * `output` - the `.gltf` or `.glb` mesh to write.
    /// * `format` - the target mesh container.
    /// * `scale` - meters per voxel, applied as a uniform scale to every vertex.
    /// * `method` - the meshing strategy.
    /// * `object` - the object's index into the document, as
    ///   [`resolve_objects`](Self::resolve_objects) returns.
    /// * `layer` - the object layer whose materials bake, a 0-based index into
    ///   the object's layers in reference order; ignored for pure geometry.
    /// * `maps` - the material maps to bake, each its own image, in order; empty
    ///   for pure geometry.
    /// * `storage` - where the baked images go.
    /// * `texture_shape` - how the baked atlas canvas is shaped around the
    ///   material texels; ignored for pure geometry.
    #[allow(clippy::too_many_arguments)]
    fn mesh_object(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        format: MeshFormat,
        scale: f64,
        method: MeshMethod,
        object: usize,
        layer: usize,
        maps: &[MeshTextureMap],
        storage: ResourceStorage,
        texture_shape: TextureShape,
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
    /// palette's index, ordered attribute keys, material count, and the objects
    /// that reference it.
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
