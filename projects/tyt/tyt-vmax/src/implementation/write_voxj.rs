use crate::{Result, VoxjEncoding, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding};
use std::path::Path;
use voxconv::{
    DependenciesImpl, ReadFormat, WriteFormat,
    ext::{load_with_ext, write_with_ext},
    voxj::{
        PositionEncoding, SampleEncoding, VoxjSerialization, VoxjWriteFormat, VoxjWriteOptions,
    },
};

/// Converts the `.vmax` package at `input` into a Voxel Json document written to
/// stdout, round-tripping through voxcore.
pub(crate) fn write_voxj(input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()> {
    // The `vmax` ext rides through boxed into the document's `ext` block.
    let state = load_with_ext(&DependenciesImpl, ReadFormat::VMax, input)?;

    let (position_encoding, sample_encoding) = block_encoding(encoding);

    let format = WriteFormat::Voxj(VoxjWriteFormat {
        serialization: serialization(format),
        options: VoxjWriteOptions {
            position_encoding,
            sample_encoding,
            ..VoxjWriteOptions::default()
        },
    });

    let files = write_with_ext(&DependenciesImpl, &format, state)?;

    let file = files.first().expect("the Voxel Json writer emits one file");

    tyt_injection::write_stdout(&file.bytes)?;

    Ok(())
}

/// Maps a CLI output form to the voxconv serialization.
fn serialization(format: VoxjFormat) -> VoxjSerialization {
    match format {
        VoxjFormat::Json => VoxjSerialization::Compact,
        VoxjFormat::PrettyJson => VoxjSerialization::Pretty,
        VoxjFormat::Zip => VoxjSerialization::Zip,
    }
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
