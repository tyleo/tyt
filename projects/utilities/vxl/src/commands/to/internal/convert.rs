use crate::{Dependencies, ObjectSelection, Result};
use std::path::Path;
use voxconv::{
    ReadFormat, WriteFormat,
    ext::{load_with_ext, save_with_ext},
};
use voxsmith::operations::to::keep_objects;

/// Converts the document at `input`, read as `from`, into the document at
/// `output`, written as `to`. A `selection` narrows the written document to
/// its objects and the hierarchy to what still places them. The source's ext
/// rides through boxed, so a same-format write rebuilds the file exactly and
/// a Voxel Json write keeps another format's ext in its `ext` block.
pub(crate) fn convert<D: Dependencies>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
    output: &Path,
    to: &WriteFormat,
    selection: &ObjectSelection,
) -> Result<()> {
    let mut state = load_with_ext(dependencies, from, input)?;

    // Without a selector the state rides through untouched. With one, the
    // pruned state is compacted because the writers index by id.
    if selection.has_selectors() {
        let object_ids = selection.resolve(&state)?;

        keep_objects(&mut state, &object_ids)?;

        state.gc();
    }

    Ok(save_with_ext(dependencies, to, state, output)?)
}
