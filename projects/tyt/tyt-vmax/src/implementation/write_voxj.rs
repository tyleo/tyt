use crate::{Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding};
use std::{io::ErrorKind, path::Path};
use voxsmith::{PositionEncoding, SampleEncoding, VoxjFileBuilder, from_vmax_package};

/// Converts the `.vmax` package at `input` into a Voxel Json document written to
/// stdout, round-tripping through voxcore.
pub(crate) fn write_voxj(input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()> {
    // The `vmax` ext carries the provenance with no native voxj home.
    let state = from_vmax_package(
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
            Err(e) => Err(e),
        },
    )?;

    // The builder keys the ext into the document's `ext` block.
    let (position, sample) = block_encoding(encoding);
    let builder = VoxjFileBuilder::new(&state)
        .position_encoding(position)
        .sample_encoding(sample);
    let bytes = match format {
        VoxjFormat::Json => builder.to_voxj_bytes()?,
        VoxjFormat::PrettyJson => builder.to_voxj_pretty_bytes()?,
        VoxjFormat::Zip => builder.to_voxjz_bytes()?,
    };

    tyt_injection::write_stdout(&bytes)?;
    Ok(())
}

/// Maps a CLI encoding choice to per-block codec encodings.
fn block_encoding(encoding: VoxjEncoding) -> (Option<PositionEncoding>, Option<SampleEncoding>) {
    match encoding {
        VoxjEncoding::Fixed { position, sample } => (
            Some(position_encoding(position)),
            Some(sample_encoding(sample)),
        ),
        VoxjEncoding::Smallest => (None, None),
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
