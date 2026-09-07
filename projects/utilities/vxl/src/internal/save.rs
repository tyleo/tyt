use crate::{Dependencies, Result};
use std::path::Path;
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, WriteFormat, codec};
use voxcore::{VoxMain, ext::VoxExtSlot};

/// Saves `state` as the document at `output`, written as `to`: the encode
/// through voxconv's impl, the files through `dependencies`.
pub(crate) fn save<D: Dependencies, T: VoxExtSlot>(
    dependencies: &D,
    to: &WriteFormat,
    state: VoxMain<T>,
    output: &Path,
) -> Result<()> {
    let files = codec::write(&VoxconvDependenciesImpl, to, state)?;

    Ok(codec::write_document_files(dependencies, output, &files)?)
}
