use crate::{Format, Result, VoxjEncoding, VoxjFormat, commands::EditState, implementation};
use std::path::Path;
use voxj_voxcore::EditStateMode;

/// Converts the voxel file at `input` into a Voxel Json document at `output`,
/// round-tripping through voxcore: the input is loaded into a
/// [`VoxMain`](voxcore::VoxMain), encoded back to a voxj document with the
/// chosen block `encoding`, then serialized in the container `format` selects.
pub fn to_voxj(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    encoding: VoxjEncoding,
    format: VoxjFormat,
    ext: bool,
    edit_state: EditState,
) -> Result<()> {
    let state = implementation::load_state_voxj(input, from)?;
    implementation::write_voxj_document(
        state,
        output,
        encoding,
        format,
        ext,
        edit_state_mode(edit_state),
    )
}

/// Maps a CLI edit-state choice to the voxj-voxcore edit-state mode.
fn edit_state_mode(edit_state: EditState) -> EditStateMode {
    match edit_state {
        EditState::Auto => EditStateMode::Auto,
        EditState::True => EditStateMode::Always,
        EditState::False => EditStateMode::Never,
    }
}
