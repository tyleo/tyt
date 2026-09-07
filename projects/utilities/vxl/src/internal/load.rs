use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, ReadFormat, codec};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Loads the document at `input`, read as `from`, into a state whose ext
/// is `T`: the files through `dependencies`, the decode through
/// voxconv's impl.
pub(crate) fn load<D: Dependencies, T: VoxExtBlockCodec>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
) -> Result<VoxMain<T>> {
    let files = codec::read_document_files(dependencies, from, input)?;

    Ok(codec::read(&VoxconvDependenciesImpl, from, &files)?)
}
