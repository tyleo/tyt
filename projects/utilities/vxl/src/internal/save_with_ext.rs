use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{
    DependenciesImpl as VoxconvDependenciesImpl, WriteFormat,
    ext::{VoxconvVoxMain, write_with_ext},
    write_document_files,
};

/// Saves `state` as the document at `output`, written as `to`: the encode
/// through voxconv's impl, the files through `dependencies`.
pub(crate) fn save_with_ext<D: Dependencies>(
    dependencies: &D,
    to: &WriteFormat,
    state: VoxconvVoxMain,
    output: &Path,
) -> Result<()> {
    let files = write_with_ext(&VoxconvDependenciesImpl, to, state)?;
    Ok(write_document_files(dependencies, output, &files)?)
}
