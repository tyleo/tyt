use crate::{Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use vmax_codec::{decode_vmax_file, from_vmax_file};
use voxj::VoxjCodecFile;
use voxj_codec::{
    PositionEncoding, SampleEncoding, encode_voxj_file, encode_voxj_file_smallest,
    to_voxj_file_bytes, to_voxj_pretty_file_bytes, to_voxjz_file_bytes,
};
use voxsmith::{vox_state_from_vmax_codec_file, voxj_codec_main_from_vox_state};

/// The voxj format version stamped on the document. A `VoxState` carries no
/// version of its own, so the writer stamps the current one.
const VOXJ_FORMAT_VERSION: u32 = 1;

/// Converts the `.vmax` package at `input` into a Voxel Json document written to
/// stdout, round-tripping through voxcore: the package is decoded, voxsmith
/// loads it into a [`VoxState`](voxcore::VoxState) and back out to the voxj
/// model, and the voxj codec block-encodes and serializes it.
pub(crate) fn write_voxj(input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()> {
    // Decode: read the whole package into a fully decoded Voxel Max document.
    let serde = from_vmax_file(
        // List every package-relative file path, descending one level into
        // subdirectories (only `QuickLook/`) so its thumbnails keep their prefix.
        || {
            let mut paths = Vec::new();
            for entry in tyt_injection::list_dir(input)? {
                let Some(name) = entry.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if entry.is_dir() {
                    for child in tyt_injection::list_dir(&entry)? {
                        if let Some(child) = child.file_name().and_then(|n| n.to_str()) {
                            paths.push(format!("{name}/{child}"));
                        }
                    }
                } else {
                    paths.push(name.to_owned());
                }
            }
            Ok(paths)
        },
        |name| match tyt_injection::read_file(&input.join(name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        },
    )?;
    let codec = decode_vmax_file(&serde)?;

    // Translate through voxcore: decoded vmax -> VoxState -> decoded voxj. The
    // `voxel-max` ext carries the provenance with no native voxj home.
    let state = vox_state_from_vmax_codec_file(&codec)?;
    let file = VoxjCodecFile {
        version: VOXJ_FORMAT_VERSION,
        main: voxj_codec_main_from_vox_state(&state),
    };

    // Encode: block-encode the document in one pass, then serialize the chosen
    // container.
    let serialized = match encoding {
        VoxjEncoding::Fixed { position, sample } => {
            encode_voxj_file(&file, position_encoding(position), sample_encoding(sample))?
        }
        VoxjEncoding::Smallest => encode_voxj_file_smallest(&file)?,
    };
    let bytes = match format {
        VoxjFormat::Json => to_voxj_file_bytes(&serialized),
        VoxjFormat::PrettyJson => to_voxj_pretty_file_bytes(&serialized),
        VoxjFormat::Zip => to_voxjz_file_bytes(&serialized),
    }
    .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    tyt_injection::write_stdout(&bytes)?;
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
