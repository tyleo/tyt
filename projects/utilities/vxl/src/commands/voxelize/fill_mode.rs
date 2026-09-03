use crate::CliValue;
use voxsmith::FillMode;

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
            FillMode::Solid => {
                "Rasterize the surface and flood-fill the volume it encloses, producing a filled \
                 body. Expects a watertight mesh"
            }
            FillMode::Surface => {
                "Rasterize only the voxels the triangles pass through, leaving a hollow shell"
            }
        }
    }
}
