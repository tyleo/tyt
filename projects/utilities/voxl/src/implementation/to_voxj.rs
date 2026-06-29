use crate::{
    EditState, Format, Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding,
};
use std::path::Path;
use voxj_codec::{
    PositionEncoding, SampleEncoding, to_voxj_file_bytes, to_voxj_pretty_file_bytes,
    to_voxjz_file_bytes,
};
use voxsmith::{EditStateMode, VoxjFileBuilder};

/// Converts the voxel file at `input` into a Voxel Json document at `output`,
/// round-tripping through voxcore: the input is loaded into a
/// [`VoxMain`](voxcore::VoxMain), encoded back to a voxj document with the
/// chosen block `encoding`, then serialized in the container `format` selects.
#[allow(clippy::too_many_arguments)]
pub(crate) fn to_voxj(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    encoding: VoxjEncoding,
    format: VoxjFormat,
    ext: bool,
    edit_state: EditState,
) -> Result<()> {
    let state = super::load_state::load_state(input, from)?;
    let file = VoxjFileBuilder::new(&state)
        .encoding(block_encoding(encoding))
        .ext(ext)
        .edit_state(edit_state_mode(edit_state))
        .build()?;
    let bytes = match format {
        VoxjFormat::Json => to_voxj_file_bytes(&file)?,
        VoxjFormat::PrettyJson => to_voxj_pretty_file_bytes(&file)?,
        VoxjFormat::Zip => to_voxjz_file_bytes(&file)?,
    };
    tyt_injection::write_file(output, &bytes)?;
    Ok(())
}

/// Maps a CLI encoding choice to a fixed codec encoding pair, or `None` for the
/// smallest per-object search.
fn block_encoding(encoding: VoxjEncoding) -> Option<(PositionEncoding, SampleEncoding)> {
    match encoding {
        VoxjEncoding::Fixed { position, sample } => {
            Some((position_encoding(position), sample_encoding(sample)))
        }
        VoxjEncoding::Smallest => None,
    }
}

/// Maps a CLI edit-state choice to the voxsmith edit-state mode.
fn edit_state_mode(edit_state: EditState) -> EditStateMode {
    match edit_state {
        EditState::Auto => EditStateMode::Auto,
        EditState::True => EditStateMode::Always,
        EditState::False => EditStateMode::Never,
    }
}

/// Maps a CLI position-encoding choice to the voxj codec encoding.
fn position_encoding(encoding: VoxjPositionEncoding) -> PositionEncoding {
    match encoding {
        VoxjPositionEncoding::RawJson => PositionEncoding::RawJson,
        VoxjPositionEncoding::BitmapBase64 => PositionEncoding::BitmapBase64,
        VoxjPositionEncoding::Hilbert => PositionEncoding::Hilbert,
    }
}

/// Maps a CLI sample-encoding choice to the voxj codec encoding.
fn sample_encoding(encoding: VoxjSampleEncoding) -> SampleEncoding {
    match encoding {
        VoxjSampleEncoding::RawJson => SampleEncoding::RawJson,
        VoxjSampleEncoding::RleJson => SampleEncoding::RleJson,
        VoxjSampleEncoding::PackedBase64 => SampleEncoding::PackedBase64,
    }
}
