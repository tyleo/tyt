use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{
    DependenciesImpl as VoxconvDependenciesImpl, WriteFormat, write, write_document_files,
};
use voxcore::{VoxMain, ext::VoxExt};

/// Saves `state` as the document at `output`, written as `to`: the encode
/// through voxconv's impl, the files through `dependencies`.
pub(crate) fn save<D: Dependencies, T: VoxExt>(
    dependencies: &D,
    to: &WriteFormat,
    state: VoxMain<T>,
    output: &Path,
) -> Result<()> {
    let files = write(&VoxconvDependenciesImpl, to, state)?;

    Ok(write_document_files(dependencies, output, &files)?)
}
