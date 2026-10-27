use crate::{CheckedExpression, CheckedProgram, Error, Expression, Result, Scalar, check_root};

/// Checks an expression like [`check_expression`](crate::check_expression),
/// settling the bare literals no operand fixes as `context`. A destination
/// whose type is known supplies it, so `1` can stand for `1.0`.
///
/// # Arguments
/// - `program`: the checked program whose end scope the expression reads.
pub fn check_expression_in_context(
    expression: &Expression,
    program: &CheckedProgram,
    context: Scalar,
) -> Result<CheckedExpression> {
    let root = check_root(&expression.root, &program.scope, Some(context)).map_err(|failure| {
        Error::Check {
            binding: None,
            failure,
        }
    })?;

    Ok(CheckedExpression { root })
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckedProgram, Dimension, Domain, Scalar, Type, TypeEnvironment, check,
        check_expression_in_context, parse, parse_expression,
    };

    fn program() -> CheckedProgram {
        let environment = TypeEnvironment {
            types: [(
                "count".to_owned(),
                Type {
                    domain: Domain::Swatch,
                    dimension: Dimension::Vec1,
                    scalar: Scalar::U8,
                },
            )]
            .into_iter()
            .collect(),
        };

        check(parse("").unwrap(), &environment).unwrap()
    }

    /// The type `text` checks to under `context`.
    fn checked_type(text: &str, context: Scalar) -> Type {
        check_expression_in_context(&parse_expression(text).unwrap(), &program(), context)
            .unwrap()
            .to_type()
    }

    fn plain(dimension: Dimension, scalar: Scalar) -> Type {
        Type {
            domain: Domain::Plain,
            dimension,
            scalar,
        }
    }

    #[test]
    fn bare_literals_settle_as_the_context() {
        assert_eq!(
            checked_type("1", Scalar::F64),
            plain(Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            checked_type("1 + 2", Scalar::U32),
            plain(Dimension::Vec1, Scalar::U32)
        );
    }

    #[test]
    fn a_fixed_type_wins_over_the_context() {
        assert_eq!(
            checked_type("1.5", Scalar::U32),
            plain(Dimension::Vec1, Scalar::F64)
        );
        assert_eq!(
            checked_type("count + 1", Scalar::F64),
            Type {
                domain: Domain::Swatch,
                dimension: Dimension::Vec1,
                scalar: Scalar::U8,
            }
        );
    }

    #[test]
    fn a_literal_outside_the_context_errors() {
        let error =
            check_expression_in_context(&parse_expression("300").unwrap(), &program(), Scalar::U8)
                .unwrap_err()
                .to_string();
        assert!(error.contains("300"), "{error}");
    }
}
