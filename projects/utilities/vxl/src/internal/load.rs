use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, ReadFormat, read, read_document_files};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Loads the document at `input`, read as `from`, into a state whose ext
/// is `T`: the files through `dependencies`, the decode through
/// voxconv's impl.
pub(crate) fn load<D: Dependencies, T: VoxExtBlockCodec>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
) -> Result<VoxMain<T>> {
    let files = read_document_files(dependencies, from, input)?;

    Ok(read(&VoxconvDependenciesImpl, from, &files)?)
}
