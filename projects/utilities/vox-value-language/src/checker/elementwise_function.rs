use crate::function::Function;

/// A function computed entry by entry over its arguments.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ElementwiseFunction {
    Abs,
    Ceil,
    Clamp,
    Cross,
    Distance,
    Dot,
    Floor,
    Length,
    Lerp,
    Max,
    Min,
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
    Smoothstep,
    Step,
}

impl ElementwiseFunction {
    /// The elementwise function a function names, if it is one.
    pub(crate) fn from_function(function: Function) -> Option<ElementwiseFunction> {
        match function {
            Function::Abs => Some(ElementwiseFunction::Abs),
            Function::Ceil => Some(ElementwiseFunction::Ceil),
            Function::Clamp => Some(ElementwiseFunction::Clamp),
            Function::Cross => Some(ElementwiseFunction::Cross),
            Function::Distance => Some(ElementwiseFunction::Distance),
            Function::Dot => Some(ElementwiseFunction::Dot),
            Function::Floor => Some(ElementwiseFunction::Floor),
            Function::Length => Some(ElementwiseFunction::Length),
            Function::Lerp => Some(ElementwiseFunction::Lerp),
            Function::Max => Some(ElementwiseFunction::Max),
            Function::Min => Some(ElementwiseFunction::Min),
            Function::Mod => Some(ElementwiseFunction::Mod),
            Function::Normalize => Some(ElementwiseFunction::Normalize),
            Function::OklabFromRgb => Some(ElementwiseFunction::OklabFromRgb),
            Function::OklchFromRgb => Some(ElementwiseFunction::OklchFromRgb),
            Function::Pow => Some(ElementwiseFunction::Pow),
            Function::R => Some(ElementwiseFunction::R),
            Function::Rg => Some(ElementwiseFunction::Rg),
            Function::Rgb => Some(ElementwiseFunction::Rgb),
            Function::Rgba => Some(ElementwiseFunction::Rgba),
            Function::RgbFromOklab => Some(ElementwiseFunction::RgbFromOklab),
            Function::RgbFromOklch => Some(ElementwiseFunction::RgbFromOklch),
            Function::Round => Some(ElementwiseFunction::Round),
            Function::Smoothstep => Some(ElementwiseFunction::Smoothstep),
            Function::Step => Some(ElementwiseFunction::Step),
            _ => None,
        }
    }

    /// The function this one names.
    pub(crate) fn to_function(self) -> Function {
        match self {
            ElementwiseFunction::Abs => Function::Abs,
            ElementwiseFunction::Ceil => Function::Ceil,
            ElementwiseFunction::Clamp => Function::Clamp,
            ElementwiseFunction::Cross => Function::Cross,
            ElementwiseFunction::Distance => Function::Distance,
            ElementwiseFunction::Dot => Function::Dot,
            ElementwiseFunction::Floor => Function::Floor,
            ElementwiseFunction::Length => Function::Length,
            ElementwiseFunction::Lerp => Function::Lerp,
            ElementwiseFunction::Max => Function::Max,
            ElementwiseFunction::Min => Function::Min,
            ElementwiseFunction::Mod => Function::Mod,
            ElementwiseFunction::Normalize => Function::Normalize,
            ElementwiseFunction::OklabFromRgb => Function::OklabFromRgb,
            ElementwiseFunction::OklchFromRgb => Function::OklchFromRgb,
            ElementwiseFunction::Pow => Function::Pow,
            ElementwiseFunction::R => Function::R,
            ElementwiseFunction::Rg => Function::Rg,
            ElementwiseFunction::Rgb => Function::Rgb,
            ElementwiseFunction::Rgba => Function::Rgba,
            ElementwiseFunction::RgbFromOklab => Function::RgbFromOklab,
            ElementwiseFunction::RgbFromOklch => Function::RgbFromOklch,
            ElementwiseFunction::Round => Function::Round,
            ElementwiseFunction::Smoothstep => Function::Smoothstep,
            ElementwiseFunction::Step => Function::Step,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{checker::ElementwiseFunction, function::Function};

    #[test]
    fn every_elementwise_function_round_trips_through_its_function() {
        for function in Function::ALL {
            if let Some(elementwise) = ElementwiseFunction::from_function(function) {
                assert_eq!(elementwise.to_function(), function);
            }
        }
    }
}
