use crate::{
    Result, VoxjColorFormat, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding,
};
use std::{fs, path::Path};
use voxcore::VoxMain;
use voxj_codec::{
    PositionEncoding, SampleEncoding, to_voxj_file_bytes, to_voxj_pretty_file_bytes,
    to_voxjz_file_bytes,
};
use voxsmith::{ColorFormat, EditStateMode, VoxjFileBuilder};

/// Encodes a voxel state into a Voxel Json document and writes it, shared by
/// every command that produces a voxj document.
///
/// # Arguments
/// * `state` - the voxel state to encode.
/// * `output` - the `.voxj` or `.voxjz` path to write.
/// * `encoding` - the per-object block encodings.
/// * `format` - the output container and printing form.
/// * `color_format` - the on-wire encoding for sRGB color pools.
/// * `ext` - when false, drops the user-defined `ext` extension block.
/// * `edit_state` - when to record each object's editor build volume.
pub fn write_voxj_document(
    state: &VoxMain,
    output: &Path,
    encoding: VoxjEncoding,
    format: VoxjFormat,
    color_format: VoxjColorFormat,
    ext: bool,
    edit_state: EditStateMode,
) -> Result<()> {
    let file = VoxjFileBuilder::new(state)
        .position_encoding(position_encoding(encoding.position))
        .sample_encoding(sample_encoding(encoding.sample))
        .color_format(match color_format {
            VoxjColorFormat::Float => ColorFormat::Float,
            VoxjColorFormat::Hex => ColorFormat::Hex,
            VoxjColorFormat::LinearFloat => ColorFormat::LinearFloat,
        })
        .ext(ext)
        .edit_state(edit_state)
        .build()?;
    let bytes = match format {
        VoxjFormat::Json => to_voxj_file_bytes(&file)?,
        VoxjFormat::PrettyJson => to_voxj_pretty_file_bytes(&file)?,
        VoxjFormat::Zip => to_voxjz_file_bytes(&file)?,
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
