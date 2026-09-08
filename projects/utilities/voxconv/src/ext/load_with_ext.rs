use crate::{
    Dependencies, ListDir, ReadFile, ReadFormat, Result, ext::read_with_ext, read_document_files,
};
use std::path::Path;
use voxcore::{VoxMain, ext::VoxExt};

/// Reads the document at `input` into a state carrying the format's ext,
/// boxed: [`read_document_files`](crate::read_document_files) then
/// [`read_with_ext`](crate::ext::read_with_ext).
pub fn load_with_ext<D: Dependencies + ReadFile + ListDir>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<VoxMain<Box<dyn VoxExt>>> {
    let files = read_document_files(dependencies, format, input)?;

    read_with_ext(dependencies, format, &files)
}
