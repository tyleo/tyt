use crate::{Dependencies, ListDir, ReadFile, ReadFormat, Result, read, read_document_files};
use meshdoc::MeshMain;
use std::path::Path;

/// Reads the document at `input` into a bare state:
/// [`read_document_files`](crate::read_document_files) then
/// [`read`](crate::read).
pub fn load<D: Dependencies + ReadFile + ListDir>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<MeshMain<()>> {
    let files = read_document_files(dependencies, format, input)?;

    read(dependencies, format, &files)
}
