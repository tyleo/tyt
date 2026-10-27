use crate::CliValue;
use voxsmith::operations::object::RenderShadow;

impl CliValue for RenderShadow {
    const VARIANTS: &'static [Self] = &[
        RenderShadow::None,
        RenderShadow::PerPixel,
        RenderShadow::PerFace,
        RenderShadow::PerCorner,
    ];

    fn name(self) -> &'static str {
        match self {
            RenderShadow::None => "none",
            RenderShadow::PerPixel => "per-pixel",
            RenderShadow::PerFace => "per-face",
            RenderShadow::PerCorner => "per-corner",
        }
    }

    fn help(self) -> &'static str {
        match self {
            RenderShadow::None => "No shadow",
            RenderShadow::PerPixel => "One shadow ray per pixel, a crisp edge across faces",
            RenderShadow::PerFace => "One shadow ray per face, a crisp staircase",
            RenderShadow::PerCorner => "One shadow ray per face corner, blended, a soft staircase",
        }
    }
}
