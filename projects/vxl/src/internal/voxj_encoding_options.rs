use crate::{VoxjEncodingPreset, VoxjPositionEncoding, VoxjSampleEncoding, cli_value_parser};
use clap::Args;
use std::path::{Path, PathBuf};
use voxconv::voxj::{PositionEncoding, SampleEncoding, VoxjSerialization, VoxjWriteOptions};

/// The voxj output container and block-encoding options, shared by commands that
/// write a voxj document.
#[derive(Clone, Debug, Args)]
pub struct VoxjEncodingOptions {
    /// Output container and printing form. Defaults to the container the
    /// output extension implies, else the input's, else compact JSON.
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
        let serialization = self.resolve_serialization(input, output.as_deref());

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

    /// Resolves the serialization from `--format`, else the extension of
    /// `output`, or of `input` when no output is given, else compact JSON.
    fn resolve_serialization(&self, input: &Path, output: Option<&Path>) -> VoxjSerialization {
        self.format
            .or_else(|| {
                let extension = output.unwrap_or(input).extension()?.to_str()?;

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

#[cfg(test)]
mod tests {
    use crate::VoxjEncodingOptions;
    use clap::Parser;
    use std::path::{Path, PathBuf};
    use voxconv::voxj::VoxjSerialization;

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        encoding_options: VoxjEncodingOptions,
    }

    fn resolve(args: &[&str], input: &str, output: Option<&str>) -> (VoxjSerialization, PathBuf) {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);

        let (serialization, _, path) = Cli::try_parse_from(argv)
            .unwrap()
            .encoding_options
            .resolve_output(Path::new(input), output.map(PathBuf::from));

        (serialization, path)
    }

    #[test]
    fn the_output_defaults_to_the_input_container() {
        assert_eq!(
            resolve(&[], "scene.voxjz", None),
            (VoxjSerialization::Zip, PathBuf::from("scene.voxjz"))
        );
        assert_eq!(
            resolve(&[], "scene.voxj", None),
            (VoxjSerialization::Compact, PathBuf::from("scene.voxj"))
        );
        assert_eq!(
            resolve(&[], "scene.vmax", None),
            (VoxjSerialization::Compact, PathBuf::from("scene.voxj"))
        );
    }

    #[test]
    fn the_output_extension_and_format_override_the_input() {
        assert_eq!(
            resolve(&[], "scene.voxjz", Some("out.voxj")).0,
            VoxjSerialization::Compact
        );
        assert_eq!(
            resolve(&["--format", "pretty"], "scene.voxjz", None),
            (VoxjSerialization::Pretty, PathBuf::from("scene.voxj"))
        );
    }
}
