use crate::{
    AttributeSelector, CameraView, ColorFormat, Dependencies, EditState, FillMode, Format,
    MeshFormat, PaletteShowLayout, ReportLayout, Result, VoxjEncoding, VoxjFormat, Width,
    implementation,
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
        side_length: Option<u32>,
        scale: Option<f64>,
        fill_mode: FillMode,
        fill_color: [u8; 4],
        encoding: VoxjEncoding,
        format: VoxjFormat,
    ) -> Result<()> {
        implementation::voxelize(
            input,
            from,
            output,
            side_length,
            scale,
            fill_mode,
            fill_color,
            encoding,
            format,
        )
    }

    fn info(&self, input: &Path, from: Option<Format>, layout: ReportLayout) -> Result<()> {
        implementation::info(input, from, layout)
    }

    fn validate(&self, input: &Path, layout: ReportLayout) -> Result<()> {
        implementation::validate(input, layout)
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

    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
        implementation::match_glob(pattern, candidates)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        implementation::write_stdout(contents)
    }
}
