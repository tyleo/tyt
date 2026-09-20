use crate::{Program, Result, lexer, parser};

/// Parses program text, a sequence of `name = expr;` statements with the
/// empty statement legal.
pub fn parse(text: &str) -> Result<Program> {
    let tokens = lexer::lex(text)?;
    let bindings = parser::parse_tokens(&tokens, text.len())?;

    Ok(Program { bindings })
}

#[cfg(test)]
mod tests {
    use crate::{Error, ParseFailure, parse};
    use std::ops::Range;

    fn render(text: &str) -> Vec<String> {
        parse(text)
            .unwrap()
            .bindings
            .into_iter()
            .map(|binding| format!("{} = {}", binding.name, binding.expression.render()))
            .collect()
    }

    fn failure(text: &str) -> (Range<usize>, ParseFailure) {
        match parse(text).unwrap_err() {
            Error::Parse { range, failure } => (range, failure),
            error => panic!("expected a parse error, found {error:?}"),
        }
    }

    #[test]
    fn a_program_is_a_sequence_of_terminated_bindings() {
        assert_eq!(
            render("tint = color.rgb; dim = tint * 0.5; bright = tint * 1.2;"),
            vec![
                "tint = (. `color` rgb)",
                "dim = (* `tint` 0.5)",
                "bright = (* `tint` 1.2)",
            ]
        );
    }

    #[test]
    fn the_empty_statement_is_legal() {
        assert_eq!(render(";"), Vec::<String>::new());
        assert_eq!(render(""), Vec::<String>::new());
        assert_eq!(render("a = 1;; b = 2;;;"), vec!["a = 1", "b = 2"]);
    }

    #[test]
    fn a_binding_requires_its_terminator() {
        assert_eq!(
            failure("a = 1"),
            (5..5, ParseFailure::UnexpectedEnd { expected: "`;`" })
        );
        assert_eq!(
            failure("a = 1 b = 2;"),
            (
                6..7,
                ParseFailure::Unexpected {
                    found: "the name `b`".to_owned(),
                    expected: "`;`"
                }
            )
        );
    }

    #[test]
    fn a_bare_expression_errors() {
        assert_eq!(failure("1 + 2;"), (0..1, ParseFailure::ExpectedBinding));
        assert_eq!(failure("a + 2;"), (0..3, ParseFailure::ExpectedBinding));
        assert_eq!(failure("a;"), (0..2, ParseFailure::ExpectedBinding));
    }

    #[test]
    fn a_reserved_name_cannot_be_bound_bare() {
        assert_eq!(
            failure("min = 1;"),
            (
                0..3,
                ParseFailure::ReservedName {
                    name: "min".to_owned()
                }
            )
        );
        assert_eq!(
            failure("true = 1;"),
            (
                0..4,
                ParseFailure::ReservedName {
                    name: "true".to_owned()
                }
            )
        );
        assert_eq!(render("`min` = 1;"), vec!["min = 1"]);
    }

    #[test]
    fn a_name_can_be_redefined() {
        assert_eq!(
            render("r2 = pow(r2, 2); r2 = r2 * 2;"),
            vec!["r2 = (pow `r2` 2)", "r2 = (* `r2` 2)"]
        );
    }

    #[test]
    fn a_missing_right_side_errors_at_its_position() {
        assert_eq!(
            failure("a = ;"),
            (
                4..5,
                ParseFailure::Unexpected {
                    found: "`;`".to_owned(),
                    expected: "an expression"
                }
            )
        );
        assert_eq!(
            failure("a ="),
            (
                3..3,
                ParseFailure::UnexpectedEnd {
                    expected: "an expression"
                }
            )
        );
    }

    #[test]
    fn a_lexing_error_surfaces_from_parse() {
        assert_eq!(failure("a = v .rg;"), (6..7, ParseFailure::DetachedPostfix));
    }
}
