use crate::{Dependencies, Result, load, save};
use std::path::Path;
use voxconv::{ReadFormat, WriteFormat};
use voxcore::{VoxMain, VoxMap};

/// Converts the document at `input`, read as `from`, into the document at
/// `output`, written as `to`. The source's ext rides through as a block, so a
/// same-format write rebuilds the file exactly and a Voxel Json write keeps
/// another format's ext in its `ext` block.
pub(crate) fn convert<D: Dependencies>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
    output: &Path,
    to: &WriteFormat,
) -> Result<()> {
    let state: VoxMain<Option<VoxMap>> = load(dependencies, input, from)?;

    save(dependencies, to, state, output)
}
