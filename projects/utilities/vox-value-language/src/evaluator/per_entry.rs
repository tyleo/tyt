use crate::evaluator::{EvalResult, Operand};

/// Computes every output entry from the operands' whole entries.
pub(crate) fn per_entry<T, U, const N: usize>(
    operands: [Operand<'_, T>; N],
    entries: usize,
    mut compute: impl FnMut([&[T]; N]) -> EvalResult<Vec<U>>,
) -> EvalResult<Vec<U>> {
    let mut output = Vec::new();

    for entry in 0..entries {
        let inputs = operands.each_ref().map(|operand| operand.entry(entry));

        output.extend(compute(inputs)?);
    }

    Ok(output)
}
