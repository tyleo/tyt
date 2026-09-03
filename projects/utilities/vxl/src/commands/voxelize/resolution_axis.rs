use crate::CliValue;
use voxsmith::ResolutionAxis;

impl CliValue for ResolutionAxis {
    const VARIANTS: &'static [Self] = &[
        ResolutionAxis::Long,
        ResolutionAxis::Short,
        ResolutionAxis::X,
        ResolutionAxis::Y,
        ResolutionAxis::Z,
    ];

    fn name(self) -> &'static str {
        match self {
            ResolutionAxis::Long => "long",
            ResolutionAxis::Short => "short",
            ResolutionAxis::X => "x",
            ResolutionAxis::Y => "y",
            ResolutionAxis::Z => "z",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ResolutionAxis::Long => "The mesh's longest extent",
            ResolutionAxis::Short => "The mesh's shortest extent",
            ResolutionAxis::X => "The x axis",
            ResolutionAxis::Y => "The y axis",
            ResolutionAxis::Z => "The z axis",
        }
    }
}
