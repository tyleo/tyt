use crate::{
    AttributeSelector, CameraView, ColorFormat, Dependencies, EditState, FillMode, Format,
    GridResolution, HierarchyViews, MaterialMode, MeshFormat, MeshMethod, PaletteListFields,
    PaletteListLayout, PaletteReduction, PaletteShowLayout, PatternView, ReportLayout, Result,
    SelectIndex, VoxjEncoding, VoxjFormat, Width, implementation,
};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn to_goxl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
        implementation::to_goxl(input, from, output)
    }

    fn to_mvox(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
        implementation::to_mvox(input, from, output)
    }

    fn to_qbcl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
        implementation::to_qbcl(input, from, output)
    }

    fn to_vmax(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        color_format: ColorFormat,
        camera: Option<CameraView>,
    ) -> Result<()> {
        implementation::to_vmax(input, from, output, color_format, camera)
    }

    fn to_voxj(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        encoding: VoxjEncoding,
        format: VoxjFormat,
        ext: bool,
        edit_state: EditState,
    ) -> Result<()> {
        implementation::to_voxj(input, from, output, encoding, format, ext, edit_state)
    }

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
    ) -> Result<()> {
        implementation::voxelize(
            input,
            from,
            output,
            resolution,
            fill_mode,
            material_mode,
            fill_color,
            name,
            reduction,
            encoding,
            format,
        )
    }

    fn resolve_objects(
        &self,
        input: &Path,
        from: Option<Format>,
        select: &[String],
        select_index: &[SelectIndex],
    ) -> Result<Vec<usize>> {
        implementation::resolve_objects(input, from, select, select_index)
    }

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
    ) -> Result<()> {
        implementation::mesh_object(input, from, output, format, scale, method, object)
    }

    fn info(&self, input: &Path, from: Option<Format>, layout: ReportLayout) -> Result<()> {
        implementation::info(input, from, layout)
    }

    fn validate(&self, input: &Path, layout: ReportLayout) -> Result<()> {
        implementation::validate(input, layout)
    }

    fn palette_list(
        &self,
        input: &Path,
        from: Option<Format>,
        filters: &[SelectIndex],
        fields: PaletteListFields,
        layout: PaletteListLayout,
    ) -> Result<()> {
        implementation::palette_list(input, from, filters, fields, layout)
    }

    fn palette_show(
        &self,
        input: &Path,
        from: Option<Format>,
        selectors: &[AttributeSelector],
        layout: PaletteShowLayout,
        width: Width,
    ) -> Result<()> {
        implementation::palette_show(input, from, selectors, layout, width)
    }

    fn hierarchy_show(
        &self,
        input: &Path,
        from: Option<Format>,
        pattern: Option<PatternView>,
        collapse_instances: bool,
        views: HierarchyViews,
    ) -> Result<()> {
        implementation::hierarchy_show(input, from, pattern, collapse_instances, views)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        implementation::write_stdout(contents)
    }
}
