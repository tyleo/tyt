use crate::{
    Components, EvalFailure, Scalar, Value,
    checker::Rounding,
    evaluator::{EvalResult, Unsigned},
};

/// Converts a value's components to another numeric type, exactly or by the
/// named rounding.
pub(crate) fn convert(
    value: &Value,
    target: Scalar,
    rounding: Option<Rounding>,
) -> EvalResult<Value> {
    let components = match (value.components(), target) {
        (Components::F32(components), Scalar::F32) => Components::F32(components.clone()),
        (Components::F32(components), Scalar::U8) => {
            Components::U8(from_f32(components, rounding, target)?)
        }
        (Components::F32(components), Scalar::U16) => {
            Components::U16(from_f32(components, rounding, target)?)
        }
        (Components::F32(components), Scalar::U32) => {
            Components::U32(from_f32(components, rounding, target)?)
        }

        (Components::U8(components), Scalar::F32) => {
            Components::F32(components.iter().map(|&value| f32::from(value)).collect())
        }
        (Components::U16(components), Scalar::F32) => {
            Components::F32(components.iter().map(|&value| f32::from(value)).collect())
        }
        (Components::U32(components), Scalar::F32) => Components::F32(
            components
                .iter()
                .map(|&value| {
                    if value <= 1 << 24 {
                        Ok(value as f32)
                    } else {
                        Err(EvalFailure::Inexact { value })
                    }
                })
                .collect::<EvalResult<_>>()?,
        ),

        (Components::U8(components), Scalar::U8) => Components::U8(components.clone()),
        (Components::U8(components), Scalar::U16) => Components::U16(between(components, target)?),
        (Components::U8(components), Scalar::U32) => Components::U32(between(components, target)?),
        (Components::U16(components), Scalar::U8) => Components::U8(between(components, target)?),
        (Components::U16(components), Scalar::U16) => Components::U16(components.clone()),
        (Components::U16(components), Scalar::U32) => Components::U32(between(components, target)?),
        (Components::U32(components), Scalar::U8) => Components::U8(between(components, target)?),
        (Components::U32(components), Scalar::U16) => Components::U16(between(components, target)?),
        (Components::U32(components), Scalar::U32) => Components::U32(components.clone()),

        (Components::Bool(_) | Components::String(_), _) | (_, Scalar::Bool | Scalar::String) => {
            unreachable!("the checker converts numbers alone")
        }
    };

    Ok(Value::new(value.domain(), value.dimension(), components)
        .expect("a conversion keeps every entry"))
}

/// Converts `f32` components into an unsigned type, exactly or by the named
/// rounding, then checks the range.
fn from_f32<T: Unsigned>(
    components: &[f32],
    rounding: Option<Rounding>,
    target: Scalar,
) -> EvalResult<Vec<T>> {
    components
        .iter()
        .map(|&value| {
            let whole = match rounding {
                None if value.fract() != 0.0 => {
                    return Err(EvalFailure::Fraction { value, target });
                }
                None => value,
                Some(Rounding::Ceil) => value.ceil(),
                Some(Rounding::Floor) => value.floor(),
                Some(Rounding::Round) => value.round(),
            };
            let out_of_range = || EvalFailure::OutOfRange {
                value: f64::from(whole),
                target,
            };

            if whole < 0.0 || f64::from(whole) > T::max_value().to_f64() {
                return Err(out_of_range());
            }

            T::from_u64(whole as u64).ok_or_else(out_of_range)
        })
        .collect()
}

/// Converts between unsigned types under a range check.
fn between<T: Unsigned, U: Unsigned>(components: &[T], target: Scalar) -> EvalResult<Vec<U>> {
    components
        .iter()
        .map(|&value| {
            U::from_u64(value.to_u64()).ok_or(EvalFailure::OutOfRange {
                value: value.to_f64(),
                target,
            })
        })
        .collect()
}
