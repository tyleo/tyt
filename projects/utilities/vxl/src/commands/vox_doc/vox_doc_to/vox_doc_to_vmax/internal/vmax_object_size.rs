use crate::CliValue;
use voxconv::vmax::VMaxObjectSize;

impl CliValue for VMaxObjectSize {
    const VARIANTS: &'static [Self] = &[
        Self::Auto,
        Self::Size32,
        Self::Size64,
        Self::Size128,
        Self::Size256,
        Self::Size512,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Size32 => "32",
            Self::Size64 => "64",
            Self::Size128 => "128",
            Self::Size256 => "256",
            Self::Size512 => "512",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Self::Auto => {
                "Keep a loaded object's extent when it fits; choose 256 or 512 for a new object"
            }
            Self::Size32 => {
                "Center each object's live voxels in a 32 x 32 x 32 workspace; error if they cannot fit"
            }
            Self::Size64 => {
                "Center each object's live voxels in a 64 x 64 x 64 workspace; error if they cannot fit"
            }
            Self::Size128 => {
                "Center each object's live voxels in a 128 x 128 x 128 workspace; error if they cannot fit"
            }
            Self::Size256 => {
                "Center each object's live voxels in a 256 x 256 x 256 workspace; error if they cannot fit"
            }
            Self::Size512 => {
                "Center each object's live voxels in a 512 x 512 x 512 workspace; error if they cannot fit"
            }
        }
    }
}
