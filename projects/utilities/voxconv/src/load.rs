use crate::{Dependencies, ListDir, ReadFile, ReadFormat, Result, read, read_document_files};
use std::path::Path;
use voxcore::VoxMain;

/// Reads the document at `input` into a bare state:
/// [`read_document_files`](crate::read_document_files) then
/// [`read`](crate::read).
pub fn load<D: Dependencies + ReadFile + ListDir>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<VoxMain<()>> {
    let files = read_document_files(dependencies, format, input)?;

    read(dependencies, format, &files)
}
