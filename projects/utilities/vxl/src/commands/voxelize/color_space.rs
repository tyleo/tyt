use crate::CliValue;
use voxsmith::ColorSpace;

impl CliValue for ColorSpace {
    const VARIANTS: &'static [Self] = &[ColorSpace::Oklab, ColorSpace::Lab, ColorSpace::Srgb];

    fn name(self) -> &'static str {
        match self {
            ColorSpace::Oklab => "oklab",
            ColorSpace::Lab => "lab",
            ColorSpace::Srgb => "srgb",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ColorSpace::Oklab => "OKLab perceptual distance",
            ColorSpace::Lab => "CIELAB distance",
            ColorSpace::Srgb => "sRGB distance",
        }
    }
}
