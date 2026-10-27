use crate::CliValue;
use voxsmith::operations::object::RenderOcclusion;

impl CliValue for RenderOcclusion {
    const VARIANTS: &'static [Self] = &[RenderOcclusion::None, RenderOcclusion::Corner];

    fn name(self) -> &'static str {
        match self {
            RenderOcclusion::None => "none",
            RenderOcclusion::Corner => "corner",
        }
    }

    fn help(self) -> &'static str {
        match self {
            RenderOcclusion::None => "No occlusion",
            RenderOcclusion::Corner => "One value per face corner from the three adjacent cells",
        }
    }
}
