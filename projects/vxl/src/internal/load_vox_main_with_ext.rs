use crate::{Dependencies, Result, warn_slashed_names};
use std::path::Path;
use voxconv::{
    ReadFormat,
    ext::{VoxconvVoxMain, load_with_ext},
};

/// Reads the document at `input` into a state carrying the format's ext:
/// [`load_with_ext()`] then [`warn_slashed_names()`].
pub fn load_vox_main_with_ext<D: Dependencies>(
    dependencies: &D,
    from: ReadFormat,
    input: &Path,
) -> Result<VoxconvVoxMain> {
    let main = load_with_ext(dependencies, from, input)?;

    warn_slashed_names(dependencies, main.state())?;

    Ok(main)
}
