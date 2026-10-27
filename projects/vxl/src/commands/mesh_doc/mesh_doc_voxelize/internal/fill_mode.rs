use crate::CliValue;
use voxsmith::utilities::FillMode;

impl CliValue for FillMode {
    const VARIANTS: &'static [Self] = &[FillMode::Solid, FillMode::Surface];

    fn name(self) -> &'static str {
        match self {
            FillMode::Solid => "solid",
            FillMode::Surface => "surface",
        }
    }

    fn help(self) -> &'static str {
        match self {
            FillMode::Solid => "Fill the body's whole volume",

            FillMode::Surface => "Keep only a hollow shell of the voxels on the body's surface",
        }
    }
}
