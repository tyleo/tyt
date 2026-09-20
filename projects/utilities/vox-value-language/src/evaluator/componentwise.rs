use crate::evaluator::{EvalResult, Operand};

/// Computes every output component from the operands' components at the
/// same position, a vec1 operand broadcasting across each entry.
pub(crate) fn componentwise<T, U, const N: usize>(
    operands: [Operand<'_, T>; N],
    entries: usize,
    width: usize,
    mut compute: impl FnMut([&T; N]) -> EvalResult<U>,
) -> EvalResult<Vec<U>> {
    let mut output = Vec::with_capacity(entries * width);

    for entry in 0..entries {
        for position in 0..width {
            let inputs = operands
                .each_ref()
                .map(|operand| operand.component(entry, position));

            output.push(compute(inputs)?);
        }
    }

    Ok(output)
}
