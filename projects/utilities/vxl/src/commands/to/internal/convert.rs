use crate::{Dependencies, ObjectSelection, Result, load, save};
use std::path::Path;
use voxconv::{ReadFormat, WriteFormat};
use voxcore::{VoxMain, VoxMap};
use voxsmith::operations::to::keep_objects;

/// Converts the document at `input`, read as `from`, into the document at
/// `output`, written as `to`. A `selection` narrows the written document to
/// its objects and the hierarchy to what still places them. The source's ext
/// rides through as a block, so a same-format write rebuilds the file exactly
/// and a Voxel Json write keeps another format's ext in its `ext` block.
pub(crate) fn convert<D: Dependencies>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
    output: &Path,
    to: &WriteFormat,
    selection: &ObjectSelection,
) -> Result<()> {
    let mut state: VoxMain<Option<VoxMap>> = load(dependencies, input, from)?;

    // Without a selector the state rides through untouched. With one, the
    // pruned state is compacted because the writers index by id.
    if selection.has_selectors() {
        let object_ids = selection.resolve(&state)?;

        keep_objects(&mut state, &object_ids)?;

        state.gc();
    }

    save(dependencies, to, state, output)
}
