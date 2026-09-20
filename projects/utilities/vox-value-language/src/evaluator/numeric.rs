use crate::{
    EvalFailure,
    evaluator::{EvalResult, Unsigned},
};

/// A numeric component type under the reductions.
pub(crate) trait Numeric: Copy + PartialOrd {
    /// The value as an `f64`.
    fn to_f64(self) -> f64;

    /// The total of the values, erroring where the type cannot hold it.
    fn sum(values: impl Iterator<Item = Self>, operation: &str) -> EvalResult<Self>;
}

impl Numeric for f32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    fn sum(values: impl Iterator<Item = Self>, operation: &str) -> EvalResult<Self> {
        let total = values.map(f64::from).sum::<f64>() as f32;

        if total.is_finite() {
            Ok(total)
        } else {
            Err(EvalFailure::NonFinite {
                operation: operation.to_owned(),
            })
        }
    }
}

impl Numeric for u8 {
    fn to_f64(self) -> f64 {
        Unsigned::to_f64(self)
    }

    fn sum(values: impl Iterator<Item = Self>, operation: &str) -> EvalResult<Self> {
        sum_unsigned(values, operation)
    }
}

impl Numeric for u16 {
    fn to_f64(self) -> f64 {
        Unsigned::to_f64(self)
    }

    fn sum(values: impl Iterator<Item = Self>, operation: &str) -> EvalResult<Self> {
        sum_unsigned(values, operation)
    }
}

impl Numeric for u32 {
    fn to_f64(self) -> f64 {
        Unsigned::to_f64(self)
    }

    fn sum(values: impl Iterator<Item = Self>, operation: &str) -> EvalResult<Self> {
        sum_unsigned(values, operation)
    }
}

/// Totals unsigned values through a `u64`, erroring where the type cannot
/// hold the total.
fn sum_unsigned<T: Unsigned>(values: impl Iterator<Item = T>, operation: &str) -> EvalResult<T> {
    let overflow = || EvalFailure::Overflow {
        operation: operation.to_owned(),
    };
    let mut total = 0u64;

    for value in values {
        total = total.checked_add(value.to_u64()).ok_or_else(overflow)?;
    }

    T::from_u64(total).ok_or_else(overflow)
}
