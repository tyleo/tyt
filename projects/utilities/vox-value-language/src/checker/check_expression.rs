use crate::{CheckedExpression, CheckedProgram, Error, Expression, Result, checker::check_root};

/// Checks an expression in the scope at a checked program's end, beside
/// every name the environment supplies.
///
/// # Arguments
/// - `program`: the checked program whose end scope the expression reads.
pub fn check_expression(
    expression: &Expression,
    program: &CheckedProgram,
) -> Result<CheckedExpression> {
    let root = check_root(&expression.root, &program.scope).map_err(|failure| Error::Check {
        binding: None,
        failure,
    })?;

    Ok(CheckedExpression { root })
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckFailure, CheckedProgram, Dimension, Domain, Error, Scalar, Type, TypeEnvironment,
        check, check_expression, parse, parse_expression,
    };

    fn program() -> CheckedProgram {
        let environment = TypeEnvironment {
            types: [(
                "occlusion".to_owned(),
                Type {
                    domain: Domain::Corner,
                    dimension: Dimension::Vec1,
                    scalar: Scalar::F32,
                },
            )]
            .into_iter()
            .collect(),
        };

        check(parse("ao = faceAvg(occlusion);").unwrap(), &environment).unwrap()
    }

    #[test]
    fn an_expression_reads_the_bindings_and_the_environment() {
        let expression = parse_expression("ao * occlusion").unwrap();

        assert_eq!(
            check_expression(&expression, &program()).unwrap().to_type(),
            Type {
                domain: Domain::Corner,
                dimension: Dimension::Vec1,
                scalar: Scalar::F32
            }
        );
    }

    #[test]
    fn a_failing_expression_names_no_binding() {
        let expression = parse_expression("ao + missing").unwrap();

        assert_eq!(
            check_expression(&expression, &program()),
            Err(Error::Check {
                binding: None,
                failure: CheckFailure::UnknownName {
                    name: "missing".to_owned()
                }
            })
        );
        assert_eq!(
            check_expression(&parse_expression("1").unwrap(), &program())
                .unwrap_err()
                .to_string(),
            "a bare whole-number literal takes its type from context, and nothing here fixes one"
        );
    }
}
