use crate::{Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use vmax_codec::from_vmax_package;
use voxj_codec::{
    PositionEncoding, SampleEncoding, to_voxj_file_bytes, to_voxj_pretty_file_bytes,
    to_voxjz_file_bytes,
};
use voxsmith::{VoxjFileBuilder, from_vmax_file};

/// Converts the `.vmax` package at `input` into a Voxel Json document written to
/// stdout, round-tripping through voxcore: the package is loaded, voxsmith
/// loads it into a [`VoxMain`](voxcore::VoxMain) and encodes it back to a voxj
/// document, which is then serialized.
pub(crate) fn write_voxj(input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()> {
    // Load: read the whole package into the lossless Voxel Max model.
    let serde = from_vmax_package(
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

    // Translate through voxcore: vmax -> VoxMain, then encode the voxj document
    // with the chosen block encodings. The `voxel-max` ext carries the
    // provenance with no native voxj home.
    let state = from_vmax_file(&serde)?;
    let serialized = VoxjFileBuilder::new(&state)
        .encoding(block_encoding(encoding))
        .build()?;
    let bytes = match format {
        VoxjFormat::Json => to_voxj_file_bytes(&serialized),
        VoxjFormat::PrettyJson => to_voxj_pretty_file_bytes(&serialized),
        VoxjFormat::Zip => to_voxjz_file_bytes(&serialized),
    }
    .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    tyt_injection::write_stdout(&bytes)?;
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
