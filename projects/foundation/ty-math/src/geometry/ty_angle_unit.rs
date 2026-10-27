/// The unit of an angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TyAngleUnit {
    /// Measures a full turn as `360`.
    Degrees,

    /// Measures a full turn as `2π`.
    Radians,
}

impl TyAngleUnit {
    /// `angle` in this unit, converted to radians.
    pub fn to_radians(self, angle: f64) -> f64 {
        match self {
            TyAngleUnit::Degrees => angle.to_radians(),
            TyAngleUnit::Radians => angle,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TyAngleUnit;
    use std::f64::consts::PI;

    #[test]
    fn converts_to_radians() {
        assert_eq!(TyAngleUnit::Degrees.to_radians(180.0), PI);
        assert_eq!(TyAngleUnit::Radians.to_radians(PI), PI);
    }
}
