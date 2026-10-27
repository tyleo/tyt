use crate::CliValue;
use ty_math::TyAngleUnit;

impl CliValue for TyAngleUnit {
    const VARIANTS: &'static [Self] = &[TyAngleUnit::Degrees, TyAngleUnit::Radians];

    fn name(self) -> &'static str {
        match self {
            TyAngleUnit::Degrees => "deg",
            TyAngleUnit::Radians => "rad",
        }
    }

    fn help(self) -> &'static str {
        match self {
            TyAngleUnit::Degrees => "Degrees",
            TyAngleUnit::Radians => "Radians",
        }
    }
}
