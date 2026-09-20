use crate::{CheckFailure, Scalar, checker::CheckResult};

/// A literal settled to its type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NumberValue {
    F32(f32),
    U8(u8),
    U16(u16),
    U32(u32),
}

impl NumberValue {
    /// Reads a literal's text as the given numeric type, erroring where the
    /// text does not fit it.
    ///
    /// # Arguments
    /// - `text`: the digits and any decimal point.
    /// - `scalar`: the numeric type the context fixed.
    pub(crate) fn parse(text: &str, scalar: Scalar) -> CheckResult<NumberValue> {
        let out_of_range = || CheckFailure::LiteralOutOfRange {
            text: text.to_owned(),
            scalar,
        };

        match scalar {
            Scalar::F32 => text
                .parse::<f32>()
                .ok()
                .filter(|value| value.is_finite())
                .map(NumberValue::F32)
                .ok_or_else(out_of_range),
            Scalar::U8 => text
                .parse()
                .map(NumberValue::U8)
                .map_err(|_| out_of_range()),
            Scalar::U16 => text
                .parse()
                .map(NumberValue::U16)
                .map_err(|_| out_of_range()),
            Scalar::U32 => text
                .parse()
                .map(NumberValue::U32)
                .map_err(|_| out_of_range()),
            Scalar::Bool | Scalar::String => Err(CheckFailure::NonNumericOperand {
                operation: "a literal".to_owned(),
                found: scalar,
            }),
        }
    }
}
