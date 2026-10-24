use crate::{Dependencies, ObjectSelection, Result, load_vox_main_with_ext};
use std::path::Path;
use voxconv::{ReadFormat, WriteFormat, ext::save_with_ext};
use voxsmith::{
    Error as VoxsmithError,
    operations::vox_doc::{keep_objects, scale_scene},
};

/// Converts the document at `input`, read as `from`, into the document at
/// `output`, written as `to`. A `selection` narrows the written document to
/// its objects and the hierarchy to what still places them. A `scene_scale`
/// then scales the scene about the world origin. The source's ext
/// rides through boxed, so a same-format write rebuilds the file exactly and
/// a Voxel Json write keeps another format's ext in its `ext` block.
pub fn convert<D: Dependencies>(
    dependencies: &D,
    input: &Path,
    from: ReadFormat,
    output: &Path,
    to: &WriteFormat,
    selection: &ObjectSelection,
    scene_scale: Option<f64>,
) -> Result<()> {
    let mut main = load_vox_main_with_ext(dependencies, from, input)?;

    // Without a selector the state rides through untouched. With one, the
    // pruned state is compacted because the writers index by id.
    if selection.has_selectors() {
        let object_ids = selection.resolve(&main)?;

        keep_objects(&mut main, &object_ids)?;

        main.gc().map_err(VoxsmithError::from)?;
    }

    if let Some(scene_scale) = scene_scale {
        scale_scene(&mut main, scene_scale)?;
    }

    Ok(save_with_ext(dependencies, to, main, output)?)
}
