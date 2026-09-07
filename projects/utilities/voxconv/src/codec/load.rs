use crate::{
    ReadFormat, Result,
    codec::{Dependencies, ListDir, ReadFile, read, read_document_files},
};
use std::path::Path;
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Reads the document at `input` into a state:
/// [`read_document_files`](crate::codec::read_document_files) then
/// [`read`](crate::codec::read).
pub fn load<D: Dependencies + ReadFile + ListDir, T: VoxExtBlockCodec>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<VoxMain<T>> {
    let files = read_document_files(dependencies, format, input)?;

    read(dependencies, format, &files)
}
