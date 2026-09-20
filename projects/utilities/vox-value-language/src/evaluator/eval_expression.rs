use crate::{CheckedExpression, Error, EvaluatedProgram, Result, Value, evaluator::eval_node};

/// Evaluates a checked expression in the scope at an evaluated program's
/// end.
pub fn eval_expression(
    expression: &CheckedExpression,
    program: &EvaluatedProgram,
) -> Result<Value> {
    eval_node(
        &expression.root,
        &program.values,
        &program.groupings,
        &program.lengths,
    )
    .map_err(|failure| Error::Eval {
        binding: None,
        failure,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        Dimension, Domain, Error, EvalFailure, check_expression, eval_expression,
        evaluator::{assert_close, bools, lamp, run},
        parse_expression,
    };

    #[test]
    fn an_expression_reads_the_bindings_and_the_environment() {
        let (checked, evaluated) = run("ao = faceAvg(computedOcclusion);", &lamp()).unwrap();
        let select = check_expression(&parse_expression("ao < 0.3").unwrap(), &checked).unwrap();

        assert_eq!(
            eval_expression(&select, &evaluated).unwrap(),
            bools(
                Domain::Face,
                &[
                    true, true, true, false, false, false, false, false, false, false
                ]
            )
        );

        let scaled = check_expression(
            &parse_expression("ao * emissiveStrength").unwrap(),
            &checked,
        )
        .unwrap();
        let value = eval_expression(&scaled, &evaluated).unwrap();

        assert_eq!(value.domain(), Domain::Face);
        assert_eq!(value.dimension(), Dimension::Vec1);
        assert_close(
            &value,
            &[0.0, 0.0, 0.0, 0.0, 0.0, 2.15, 2.55, 2.95, 3.35, 3.75],
        );
    }

    #[test]
    fn a_failing_expression_names_no_binding() {
        let (checked, evaluated) = run("", &lamp()).unwrap();
        let expression =
            check_expression(&parse_expression("count[5]").unwrap(), &checked).unwrap();

        assert_eq!(
            eval_expression(&expression, &evaluated),
            Err(Error::Eval {
                binding: None,
                failure: EvalFailure::IndexOutOfRange {
                    index: 5,
                    entries: 2
                }
            })
        );
        assert_eq!(
            eval_expression(&expression, &evaluated)
                .unwrap_err()
                .to_string(),
            "index 5 reaches past 2 entries"
        );
    }
}
