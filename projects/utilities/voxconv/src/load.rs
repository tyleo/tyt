use crate::{Dependencies, ListDir, ReadFile, ReadFormat, Result, read, read_document_files};
use std::path::Path;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Reads the document at `input` into a state:
/// [`read_document_files`](crate::read_document_files) then
/// [`read`](crate::read).
pub fn load<D: Dependencies + ReadFile + ListDir, T: VoxExtBlockCodec>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<VoxMain<T>> {
    let files = read_document_files(dependencies, format, input)?;

    read(dependencies, format, &files)
}
