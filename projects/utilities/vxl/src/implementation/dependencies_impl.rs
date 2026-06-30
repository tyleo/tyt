use crate::{
    CameraView, ColorFormat, Dependencies, EditState, Format, Result, VoxjEncoding, VoxjFormat,
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

    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
        implementation::match_glob(pattern, candidates)
    }
}
