use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{
    DependenciesImpl as VoxconvDependenciesImpl, ReadFormat, ext::read_with_ext,
    read_document_files,
};
use voxcore::{VoxMain, ext::VoxExt};

/// Loads the document at `input`, read as `from`, into a state carrying the
/// format's ext, boxed: the files through `dependencies`, the decode through
/// voxconv's impl.
pub(crate) fn load_with_ext<D: Dependencies>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
) -> Result<VoxMain<Box<dyn VoxExt>>> {
    let files = read_document_files(dependencies, from, input)?;

    Ok(read_with_ext(&VoxconvDependenciesImpl, from, &files)?)
}
