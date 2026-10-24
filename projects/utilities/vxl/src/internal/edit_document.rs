use crate::{Dependencies, Result, VoxelInput, VoxjOutput, load_vox_main_with_ext};
use voxconv::ext::{VoxconvVoxMain, save_with_ext};
use voxsmith::Error as VoxsmithError;

/// The loaded ext rides through boxed into the voxj `ext` block.
pub fn edit_document<D: Dependencies>(
    dependencies: &D,
    input: &VoxelInput,
    output: VoxjOutput,
    edit: impl FnOnce(&mut VoxconvVoxMain) -> Result<()>,
) -> Result<()> {
    let from = input.resolve_format()?;

    let (format, output) = output.resolve(&input.path);

    let mut main = load_vox_main_with_ext(dependencies, from, &input.path)?;

    edit(&mut main)?;

    // The writer writes each id as an array index, which needs the holes the
    // edit left compacted.
    main.gc().map_err(VoxsmithError::from)?;

    Ok(save_with_ext(dependencies, &format, main, &output)?)
}
