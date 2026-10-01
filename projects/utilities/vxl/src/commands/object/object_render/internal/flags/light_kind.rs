use crate::CliValue;

/// A light's kind, which decides the elements it takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LightKind {
    /// A rotation shining down its -Z from infinitely far away.
    Directional,

    /// A position with inverse-square falloff and an optional range.
    Point,

    /// A position shining down its -Z within a cone, falling off as a point
    /// light does.
    Spot,

    /// The ambient term, a sky color above and a ground color below.
    Hemisphere,
}

impl CliValue for LightKind {
    const VARIANTS: &'static [Self] = &[
        LightKind::Directional,
        LightKind::Point,
        LightKind::Spot,
        LightKind::Hemisphere,
    ];

    fn name(self) -> &'static str {
        match self {
            LightKind::Directional => "directional",
            LightKind::Point => "point",
            LightKind::Spot => "spot",
            LightKind::Hemisphere => "hemisphere",
        }
    }

    fn help(self) -> &'static str {
        match self {
            LightKind::Directional => "A rotation shining down its -Z from infinitely far away",
            LightKind::Point => "A position with inverse-square falloff and an optional range",
            LightKind::Spot => {
                "A position shining down its -Z within a cone, falling off as a point light does"
            }
            LightKind::Hemisphere => "The ambient term, a sky color above and a ground color below",
        }
    }
}
