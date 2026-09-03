use crate::CliValue;
use voxsmith::Dither;

impl CliValue for Dither {
    const VARIANTS: &'static [Self] = &[Dither::None, Dither::FloydSteinberg, Dither::Ordered];

    fn name(self) -> &'static str {
        match self {
            Dither::None => "none",
            Dither::FloydSteinberg => "floyd-steinberg",
            Dither::Ordered => "ordered",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Dither::None => "No diffusion; snap each sample to the nearest value",
            Dither::FloydSteinberg => "Floyd-Steinberg diffusion in 3D voxel order",
            Dither::Ordered => "Ordered, threshold-matrix dithering",
        }
    }
}
