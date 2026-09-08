use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{
    DependenciesImpl as VoxconvDependenciesImpl, WriteFormat, ext::write_with_ext,
    write_document_files,
};
use voxcore::{VoxMain, ext::VoxExt};

/// Saves `state` as the document at `output`, written as `to`: the encode
/// through voxconv's impl, the files through `dependencies`.
pub(crate) fn save_with_ext<D: Dependencies>(
    dependencies: &D,
    to: &WriteFormat,
    state: VoxMain<Box<dyn VoxExt>>,
    output: &Path,
) -> Result<()> {
    let files = write_with_ext(&VoxconvDependenciesImpl, to, state)?;

    Ok(write_document_files(dependencies, output, &files)?)
}
