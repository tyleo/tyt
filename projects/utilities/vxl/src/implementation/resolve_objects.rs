use crate::{Format, Result, implementation};
use std::path::Path;
use voxsmith::{IndexRange, select_objects};

/// Loads the voxel file at `input` and resolves the object selectors against
/// it with [`select_objects`].
pub fn resolve_objects(
    input: &Path,
    from: Option<Format>,
    select: &[String],
    select_index: &[IndexRange],
) -> Result<Vec<usize>> {
    let state = implementation::load_state(input, from)?;

    Ok(select_objects(&state, select, select_index)?)
}
