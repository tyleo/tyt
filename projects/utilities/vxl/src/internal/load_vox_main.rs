use crate::{Dependencies, Result, warn_slashed_names};
use std::path::Path;
use voxconv::{ReadFormat, load};
use voxcore::VoxMain;

/// Reads the document at `input` into a bare state: [`load()`] then
/// [`warn_slashed_names()`].
pub fn load_vox_main<D: Dependencies>(
    dependencies: &D,
    from: ReadFormat,
    input: &Path,
) -> Result<VoxMain> {
    let main = load(dependencies, from, input)?;

    warn_slashed_names(dependencies, main.state())?;

    Ok(main)
}
