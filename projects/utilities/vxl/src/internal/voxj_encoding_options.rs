use crate::{VoxjEncodingPreset, VoxjPositionEncoding, VoxjSampleEncoding, cli_value_parser};
use clap::Args;
use std::path::{Path, PathBuf};
use voxconv::voxj::{PositionEncoding, SampleEncoding, VoxjSerialization, VoxjWriteOptions};

/// The voxj output container and block-encoding options, shared by commands that
/// write a voxj document.
#[derive(Clone, Debug, Args)]
pub struct VoxjEncodingOptions {
    /// Output container and printing form. Defaults to compact JSON, or the
    /// container inferred from the output extension.
    #[arg(
        value_name = "format",
        long,
        value_parser = cli_value_parser::<VoxjSerialization>()
    )]
    format: Option<VoxjSerialization>,

    /// Default block-encoding strategy. Per-block flags override it.
    #[arg(value_name = "encoding-preset", long)]
    encoding_preset: Option<VoxjEncodingPreset>,

    /// Position-block encoding. Follows `--encoding-preset` when unset.
    #[arg(value_name = "position-encoding", long)]
    position_encoding: Option<VoxjPositionEncoding>,

    /// Sample-block encoding. Follows `--encoding-preset` when unset.
    #[arg(value_name = "sample-encoding", long)]
    sample_encoding: Option<VoxjSampleEncoding>,
}

impl VoxjEncodingOptions {
    /// Resolves the write target and its path together because the
    /// serialization decides the extension. The ext block and edit state keep
    /// their defaults for the command to set.
    pub fn resolve_output(
        &self,
        input: &Path,
        output: Option<PathBuf>,
    ) -> (VoxjSerialization, VoxjWriteOptions, PathBuf) {
        let serialization = self.resolve_serialization(output.as_deref());

        let path = output.unwrap_or_else(|| input.with_extension(serialization.extension()));

        let position = self
            .position_encoding
            .unwrap_or_else(|| default_position(self.encoding_preset));

        let sample = self
            .sample_encoding
            .unwrap_or_else(|| default_sample(self.encoding_preset));

        let options = VoxjWriteOptions {
            position_encoding: position_encoding(position),
            sample_encoding: sample_encoding(sample),
            ..VoxjWriteOptions::default()
        };

        (serialization, options, path)
    }

    /// Resolves the serialization from `--format`, else `output`'s extension,
    /// else compact JSON.
    fn resolve_serialization(&self, output: Option<&Path>) -> VoxjSerialization {
        self.format
            .or_else(|| {
                let extension = output?.extension()?.to_str()?;

                VoxjSerialization::from_extension(extension)
            })
            .unwrap_or_default()
    }
}

/// Position encoding for `preset`, used when `--position-encoding` is unset.
fn default_position(preset: Option<VoxjEncodingPreset>) -> VoxjPositionEncoding {
    match preset {
        None | Some(VoxjEncodingPreset::Size) => VoxjPositionEncoding::Smallest,
        Some(VoxjEncodingPreset::Fast) => VoxjPositionEncoding::BitmapBase64,
        Some(VoxjEncodingPreset::Pretty) => VoxjPositionEncoding::RawJson,
    }
}

/// Sample encoding for `preset`, used when `--sample-encoding` is unset.
fn default_sample(preset: Option<VoxjEncodingPreset>) -> VoxjSampleEncoding {
    match preset {
        None | Some(VoxjEncodingPreset::Size) => VoxjSampleEncoding::Smallest,
        Some(VoxjEncodingPreset::Fast) => VoxjSampleEncoding::PackedBase64,
        Some(VoxjEncodingPreset::Pretty) => VoxjSampleEncoding::RawJson,
    }
}

/// The codec encoding a position-encoding choice selects, or `None` for the
/// smallest search.
fn position_encoding(encoding: VoxjPositionEncoding) -> Option<PositionEncoding> {
    match encoding {
        VoxjPositionEncoding::Smallest => None,
        VoxjPositionEncoding::RawJson => Some(PositionEncoding::RawJson),
        VoxjPositionEncoding::BitmapBase64 => Some(PositionEncoding::BitmapBase64),
        VoxjPositionEncoding::Hilbert => Some(PositionEncoding::Hilbert),
    }
}

/// The codec encoding a sample-encoding choice selects, or `None` for the
/// smallest search.
fn sample_encoding(encoding: VoxjSampleEncoding) -> Option<SampleEncoding> {
    match encoding {
        VoxjSampleEncoding::Smallest => None,
        VoxjSampleEncoding::RawJson => Some(SampleEncoding::RawJson),
        VoxjSampleEncoding::RleJson => Some(SampleEncoding::RleJson),
        VoxjSampleEncoding::PackedBase64 => Some(SampleEncoding::PackedBase64),
    }
}
