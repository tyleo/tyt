use crate::{ColorFormat, Dependencies, Format, Result, VoxjEncoding, VoxjFormat};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn to_goxl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
        super::to_goxl::to_goxl(input, from, output)
    }

    fn to_mvox(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
        super::to_mvox::to_mvox(input, from, output)
    }

    fn to_qbcl(&self, input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
        super::to_qbcl::to_qbcl(input, from, output)
    }

    fn to_vmax(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        color_format: ColorFormat,
    ) -> Result<()> {
        super::to_vmax::to_vmax(input, from, output, color_format)
    }

    fn to_voxj(
        &self,
        input: &Path,
        from: Option<Format>,
        output: &Path,
        encoding: VoxjEncoding,
        format: VoxjFormat,
        ext: bool,
    ) -> Result<()> {
        super::to_voxj::to_voxj(input, from, output, encoding, format, ext)
    }
}
