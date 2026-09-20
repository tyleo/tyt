/// A reserved function name.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Function {
    Abs,
    All,
    Any,
    Avg,
    Ceil,
    CeilU8,
    CeilU16,
    CeilU32,
    Clamp,
    Corner,
    Cross,
    Default,
    Distance,
    Dot,
    F32,
    Face,
    FaceAvg,
    FaceMax,
    FaceMin,
    FaceSum,
    Floor,
    FloorU8,
    FloorU16,
    FloorU32,
    Length,
    Lerp,
    Max,
    Min,
    Mix,
    Mod,
    Normalize,
    OklabFromRgb,
    OklchFromRgb,
    Pow,
    R,
    Rg,
    Rgb,
    Rgba,
    RgbFromOklab,
    RgbFromOklch,
    Round,
    RoundU8,
    RoundU16,
    RoundU32,
    Smoothstep,
    Step,
    Sum,
    Swatch,
    SwatchAvg,
    SwatchMax,
    SwatchMin,
    SwatchSum,
    U8,
    U16,
    U32,
    Voxel,
    VoxelAvg,
    VoxelMax,
    VoxelMin,
    VoxelSum,
}

impl Function {
    /// Every function, in reserved-name order.
    pub(crate) const ALL: [Function; 60] = [
        Function::R,
        Function::Rg,
        Function::Rgb,
        Function::Rgba,
        Function::Min,
        Function::Max,
        Function::Sum,
        Function::Avg,
        Function::Any,
        Function::All,
        Function::Abs,
        Function::Pow,
        Function::Mod,
        Function::Clamp,
        Function::Lerp,
        Function::Mix,
        Function::Step,
        Function::Smoothstep,
        Function::Floor,
        Function::Ceil,
        Function::Round,
        Function::F32,
        Function::U8,
        Function::U16,
        Function::U32,
        Function::CeilU8,
        Function::CeilU16,
        Function::CeilU32,
        Function::FloorU8,
        Function::FloorU16,
        Function::FloorU32,
        Function::RoundU8,
        Function::RoundU16,
        Function::RoundU32,
        Function::Dot,
        Function::Length,
        Function::Distance,
        Function::Normalize,
        Function::Cross,
        Function::OklabFromRgb,
        Function::RgbFromOklab,
        Function::OklchFromRgb,
        Function::RgbFromOklch,
        Function::FaceAvg,
        Function::FaceMin,
        Function::FaceMax,
        Function::FaceSum,
        Function::VoxelAvg,
        Function::VoxelMin,
        Function::VoxelMax,
        Function::VoxelSum,
        Function::SwatchAvg,
        Function::SwatchMin,
        Function::SwatchMax,
        Function::SwatchSum,
        Function::Swatch,
        Function::Voxel,
        Function::Face,
        Function::Corner,
        Function::Default,
    ];

    /// The function a reserved name spells, if any.
    pub(crate) fn from_name(name: &str) -> Option<Function> {
        Function::ALL
            .into_iter()
            .find(|function| function.name() == name)
    }

    /// The reserved name.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Function::Abs => "abs",
            Function::All => "all",
            Function::Any => "any",
            Function::Avg => "avg",
            Function::Ceil => "ceil",
            Function::CeilU8 => "ceil_u8",
            Function::CeilU16 => "ceil_u16",
            Function::CeilU32 => "ceil_u32",
            Function::Clamp => "clamp",
            Function::Corner => "corner",
            Function::Cross => "cross",
            Function::Default => "default",
            Function::Distance => "distance",
            Function::Dot => "dot",
            Function::F32 => "f32",
            Function::Face => "face",
            Function::FaceAvg => "faceAvg",
            Function::FaceMax => "faceMax",
            Function::FaceMin => "faceMin",
            Function::FaceSum => "faceSum",
            Function::Floor => "floor",
            Function::FloorU8 => "floor_u8",
            Function::FloorU16 => "floor_u16",
            Function::FloorU32 => "floor_u32",
            Function::Length => "length",
            Function::Lerp => "lerp",
            Function::Max => "max",
            Function::Min => "min",
            Function::Mix => "mix",
            Function::Mod => "mod",
            Function::Normalize => "normalize",
            Function::OklabFromRgb => "oklabFromRgb",
            Function::OklchFromRgb => "oklchFromRgb",
            Function::Pow => "pow",
            Function::R => "r",
            Function::Rg => "rg",
            Function::Rgb => "rgb",
            Function::Rgba => "rgba",
            Function::RgbFromOklab => "rgbFromOklab",
            Function::RgbFromOklch => "rgbFromOklch",
            Function::Round => "round",
            Function::RoundU8 => "round_u8",
            Function::RoundU16 => "round_u16",
            Function::RoundU32 => "round_u32",
            Function::Smoothstep => "smoothstep",
            Function::Step => "step",
            Function::Sum => "sum",
            Function::Swatch => "swatch",
            Function::SwatchAvg => "swatchAvg",
            Function::SwatchMax => "swatchMax",
            Function::SwatchMin => "swatchMin",
            Function::SwatchSum => "swatchSum",
            Function::U8 => "u8",
            Function::U16 => "u16",
            Function::U32 => "u32",
            Function::Voxel => "voxel",
            Function::VoxelAvg => "voxelAvg",
            Function::VoxelMax => "voxelMax",
            Function::VoxelMin => "voxelMin",
            Function::VoxelSum => "voxelSum",
        }
    }

    /// The argument counts the function takes.
    pub(crate) fn argument_counts(self) -> &'static [usize] {
        match self {
            Function::Min | Function::Max => &[1, 2],

            Function::Rg
            | Function::Pow
            | Function::Mod
            | Function::Step
            | Function::Dot
            | Function::Distance
            | Function::Cross
            | Function::Default => &[2],

            Function::Rgb
            | Function::Clamp
            | Function::Lerp
            | Function::Mix
            | Function::Smoothstep => &[3],

            Function::Rgba => &[4],
            _ => &[1],
        }
    }

    /// The argument counts, spelled for an error.
    pub(crate) fn arity(self) -> String {
        self.argument_counts()
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" or ")
    }

    /// Whether the function takes this many arguments.
    pub(crate) fn takes(self, count: usize) -> bool {
        self.argument_counts().contains(&count)
    }
}

#[cfg(test)]
mod tests {
    use crate::function::Function;

    #[test]
    fn every_name_round_trips() {
        for function in Function::ALL {
            assert_eq!(Function::from_name(function.name()), Some(function));
        }
    }

    #[test]
    fn an_unreserved_name_is_no_function() {
        assert_eq!(Function::from_name("albedo"), None);
        assert_eq!(Function::from_name("Min"), None);
    }

    #[test]
    fn min_and_max_take_one_or_two_arguments() {
        assert!(Function::Min.takes(1));
        assert!(Function::Min.takes(2));
        assert!(!Function::Min.takes(3));
        assert_eq!(Function::Min.arity(), "1 or 2");
        assert!(Function::Rgba.takes(4));
        assert!(!Function::Rgba.takes(3));
        assert_eq!(Function::Rgba.arity(), "4");
    }
}
