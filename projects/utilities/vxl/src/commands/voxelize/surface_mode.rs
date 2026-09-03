use crate::CliValue;
use voxsmith::SurfaceMode;

impl CliValue for SurfaceMode {
    const VARIANTS: &'static [Self] = &[SurfaceMode::CenterInside, SurfaceMode::TriangleCover];

    fn name(self) -> &'static str {
        match self {
            SurfaceMode::CenterInside => "center-inside",
            SurfaceMode::TriangleCover => "triangle-cover",
        }
    }

    fn help(self) -> &'static str {
        match self {
            SurfaceMode::CenterInside => {
                "Fill a cell when its center lies inside the surface. Expects a closed mesh"
            }
            SurfaceMode::TriangleCover => {
                "Fill a cell when any triangle passes through it. Handles an open mesh, but marks \
                 both sides of a face that lands on a cell boundary"
            }
        }
    }
}
