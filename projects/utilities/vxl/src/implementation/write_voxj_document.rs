use crate::{Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding};
use std::{fs, path::Path};
use voxsmith::{
    EditStateMode, PositionEncoding, SampleEncoding, VoxjFileBuilder,
    voxcore::{VoxMain, ext::VoxExtSlot},
};

/// Encodes a voxel state into a Voxel Json document and writes it, shared by
/// every command that produces a voxj document. The builder encodes the
/// state's ext slot into the document's `ext` block, so any format's ext
/// rides along.
///
/// # Arguments
/// * `state` - the voxel state to encode.
/// * `output` - the `.voxj` or `.voxjz` path to write.
/// * `encoding` - the per-object block encodings.
/// * `format` - the output container and printing form.
/// * `ext` - when false, drops the user-defined `ext` extension block.
/// * `edit_state` - when to record each object's editor build volume.
pub fn write_voxj_document<T: VoxExtSlot>(
    state: VoxMain<T>,
    output: &Path,
    encoding: VoxjEncoding,
    format: VoxjFormat,
    ext: bool,
    edit_state: EditStateMode,
) -> Result<()> {
    let builder = VoxjFileBuilder::new(&state)
        .position_encoding(position_encoding(encoding.position))
        .sample_encoding(sample_encoding(encoding.sample))
        .ext(ext)
        .edit_state(edit_state);

    let bytes = match format {
        VoxjFormat::Json => builder.to_voxj_bytes()?,
        VoxjFormat::PrettyJson => builder.to_voxj_pretty_bytes()?,
        VoxjFormat::Zip => builder.to_voxjz_bytes()?,
    };

    fs::write(output, &bytes)?;

    Ok(())
}

/// Maps a CLI position-encoding choice to the voxj codec encoding.
fn position_encoding(encoding: VoxjPositionEncoding) -> Option<PositionEncoding> {
    match encoding {
        VoxjPositionEncoding::Smallest => None,
        VoxjPositionEncoding::RawJson => Some(PositionEncoding::RawJson),
        VoxjPositionEncoding::BitmapBase64 => Some(PositionEncoding::BitmapBase64),
        VoxjPositionEncoding::Hilbert => Some(PositionEncoding::Hilbert),
    }
}

/// Maps a CLI sample-encoding choice to the voxj codec encoding.
fn sample_encoding(encoding: VoxjSampleEncoding) -> Option<SampleEncoding> {
    match encoding {
        VoxjSampleEncoding::Smallest => None,
        VoxjSampleEncoding::RawJson => Some(SampleEncoding::RawJson),
        VoxjSampleEncoding::RleJson => Some(SampleEncoding::RleJson),
        VoxjSampleEncoding::PackedBase64 => Some(SampleEncoding::PackedBase64),
    }
}
