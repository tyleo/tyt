use crate::CliValue;
use voxconv::voxj::VoxjSerialization;

impl CliValue for VoxjSerialization {
    const VARIANTS: &'static [Self] = &[
        VoxjSerialization::Compact,
        VoxjSerialization::Zip,
        VoxjSerialization::Pretty,
    ];

    fn name(self) -> &'static str {
        match self {
            VoxjSerialization::Compact => "json",
            VoxjSerialization::Zip => "zip",
            VoxjSerialization::Pretty => "pretty",
        }
    }

    fn help(self) -> &'static str {
        match self {
            VoxjSerialization::Compact => "Compact `.voxj` JSON",
            VoxjSerialization::Zip => "Compressed `.voxjz` zip archive",
            VoxjSerialization::Pretty => "Pretty-printed `.voxj` JSON",
        }
    }
}
