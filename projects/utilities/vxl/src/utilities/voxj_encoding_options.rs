use crate::{
    VoxjEncoding, VoxjEncodingPreset, VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding,
};
use clap::Args;
use std::path::Path;

/// The voxj output container and block-encoding options, shared by commands that
/// write a voxj document.
#[derive(Clone, Debug, Args)]
pub struct VoxjEncodingOptions {
    /// Output container and printing form. Defaults to compact JSON, or the
    /// container inferred from the output extension.
    #[arg(value_name = "format", long)]
    format: Option<VoxjFormat>,

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
    /// Resolves the per-block encoding, filling unset blocks from the preset.
    pub fn encoding(&self) -> VoxjEncoding {
        VoxjEncoding {
            position: self
                .position_encoding
                .unwrap_or_else(|| default_position(self.encoding_preset)),
            sample: self
                .sample_encoding
                .unwrap_or_else(|| default_sample(self.encoding_preset)),
        }
    }

    /// Resolves the output container from `--format`, else `output`'s extension,
    /// else compact JSON.
    pub fn resolve_format(&self, output: Option<&Path>) -> VoxjFormat {
        self.format
            .or_else(|| output.and_then(VoxjFormat::from_path))
            .unwrap_or(VoxjFormat::Json)
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
