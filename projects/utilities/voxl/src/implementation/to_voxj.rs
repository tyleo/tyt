use crate::{Format, Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding};
use std::path::Path;
use voxj_codec::{
    PositionEncoding, SampleEncoding, to_voxj_file_bytes, to_voxj_pretty_file_bytes,
    to_voxjz_file_bytes,
};
use voxsmith::{to_voxj_file, to_voxj_file_with};

/// Converts the voxel file at `input` into a Voxel Json document at `output`,
/// round-tripping through voxcore: the input is loaded into a
/// [`VoxMain`](voxcore::VoxMain), encoded back to a voxj document with the
/// chosen block `encoding`, then serialized in the container `format` selects.
/// When `ext` is false, the user-defined `ext` extension block is omitted from
/// the output.
pub(crate) fn to_voxj(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    encoding: VoxjEncoding,
    format: VoxjFormat,
    ext: bool,
) -> Result<()> {
    let state = super::load_state::load_state(input, from)?;
    let mut file = match encoding {
        VoxjEncoding::Fixed { position, sample } => {
            to_voxj_file_with(&state, position_encoding(position), sample_encoding(sample))?
        }
        VoxjEncoding::Smallest => to_voxj_file(&state)?,
    };
    if !ext {
        file.main.ext = None;
    }
    let bytes = match format {
        VoxjFormat::Json => to_voxj_file_bytes(&file)?,
        VoxjFormat::PrettyJson => to_voxj_pretty_file_bytes(&file)?,
        VoxjFormat::Zip => to_voxjz_file_bytes(&file)?,
    };
    tyt_injection::write_file(output, &bytes)?;
    Ok(())
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
