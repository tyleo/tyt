use crate::CliValue;
use ty_math::TyAxis3;

impl CliValue for TyAxis3 {
    const VARIANTS: &'static [Self] = &[TyAxis3::X, TyAxis3::Y, TyAxis3::Z];

    fn name(self) -> &'static str {
        match self {
            TyAxis3::X => "x",
            TyAxis3::Y => "y",
            TyAxis3::Z => "z",
        }
    }

    fn help(self) -> &'static str {
        match self {
            TyAxis3::X => "The x-axis",
            TyAxis3::Y => "The y-axis, up",
            TyAxis3::Z => "The z-axis, toward the viewer",
        }
    }
}
