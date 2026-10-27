use crate::CliValue;
use voxsmith::utilities::PropertyInterpretation;

impl CliValue for PropertyInterpretation {
    const VARIANTS: &'static [Self] = &[
        PropertyInterpretation::Auto,
        PropertyInterpretation::LinearColor,
        PropertyInterpretation::SrgbColor,
        PropertyInterpretation::Numeric,
    ];

    fn name(self) -> &'static str {
        match self {
            PropertyInterpretation::Auto => "auto",
            PropertyInterpretation::LinearColor => "linear-color",
            PropertyInterpretation::Numeric => "numeric",
            PropertyInterpretation::SrgbColor => "srgb-color",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PropertyInterpretation::Auto => {
                "`baseColor` and `emissiveColor` as `linear-color`, any other property as `numeric`"
            }

            PropertyInterpretation::LinearColor => "A three- or four-float vector of linear light",

            PropertyInterpretation::Numeric => "Raw components of any float, int, or vector",

            PropertyInterpretation::SrgbColor => {
                "A three- or four-float vector of sRGB-encoded color"
            }
        }
    }
}
