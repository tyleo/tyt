use crate::{
    Dependencies, Format, MeshFormat, Result, VoxjEncoding, VoxjFormat, Width,
    commands::{
        CameraView, ColorFormat, EditState, FillMode, GridResolution, HierarchyShowLayout,
        HierarchyViews, MaterialMode, MeshMethod, MeshTextureMap, OutOfRangeProperty,
        PaletteReduction, PatternView, ResourceStorage, SurfaceMode, TextureShape,
    },
    implementation,
};
use std::{num::NonZeroU8, path::Path};
use voxsmith::{
    IndexRange, InfoLayout, PaletteListFields, PaletteListLayout, PaletteShowLabel,
    PaletteShowLayout, PaletteShowTableShape, PropertySelector, ValidateLayout,
};

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
        surface_mode: SurfaceMode,
        fill_mode: FillMode,
        material_mode: MaterialMode,
        fill_color: Option<[u8; 4]>,
        name: Option<&str>,
        reduction: PaletteReduction,
        encoding: VoxjEncoding,
        format: VoxjFormat,
        out_of_range_property: OutOfRangeProperty,
    ) -> Result<()> {
        implementation::voxelize(
            input,
            from,
            output,
            resolution,
            surface_mode,
            fill_mode,
            material_mode,
            fill_color,
            name,
            reduction,
            encoding,
            format,
            out_of_range_property,
        )
    }

    fn resolve_objects(
        &self,
        input: &Path,
        from: Option<Format>,
        select: &[String],
        select_index: &[IndexRange],
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
        object_index: usize,
        maps: &[MeshTextureMap],
        storage: ResourceStorage,
        texture_shape: TextureShape,
    ) -> Result<()> {
        implementation::mesh_object(
            input,
            from,
            output,
            format,
            scale,
            method,
            object_index,
            maps,
            storage,
            texture_shape,
        )
    }

    fn info(&self, input: &Path, from: Option<Format>, layout: InfoLayout) -> Result<()> {
        implementation::info(input, from, layout)
    }

    fn validate(&self, input: &Path, layout: ValidateLayout) -> Result<()> {
        implementation::validate(input, layout)
    }

    fn palette_list(
        &self,
        input: &Path,
        from: Option<Format>,
        filters: &[IndexRange],
        fields: PaletteListFields,
        layout: PaletteListLayout,
    ) -> Result<()> {
        implementation::palette_list(input, from, filters, fields, layout)
    }

    fn palette_show(
        &self,
        input: &Path,
        from: Option<Format>,
        selectors: &[PropertySelector],
        layout: PaletteShowLayout,
        label: Option<PaletteShowLabel>,
        header_level: Option<NonZeroU8>,
        table_shape: Option<PaletteShowTableShape>,
        width: Width,
    ) -> Result<()> {
        implementation::palette_show(
            input,
            from,
            selectors,
            layout,
            label,
            header_level,
            table_shape,
            width,
        )
    }

    fn hierarchy_show(
        &self,
        input: &Path,
        from: Option<Format>,
        pattern: Option<PatternView>,
        layout: HierarchyShowLayout,
        collapse_instances: bool,
        views: HierarchyViews,
    ) -> Result<()> {
        implementation::hierarchy_show(input, from, pattern, layout, collapse_instances, views)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        implementation::write_stdout(contents)
    }
}
