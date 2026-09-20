use crate::{Expression, Result, lexer, parser};

/// Parses one expression spanning the whole text.
pub fn parse_expression(text: &str) -> Result<Expression> {
    let tokens = lexer::lex(text)?;
    let root = parser::parse_expression_tokens(&tokens, text.len())?;

    Ok(Expression { root })
}

#[cfg(test)]
mod tests {
    use crate::{Error, ParseFailure, parse_expression};
    use std::ops::Range;

    fn render(text: &str) -> String {
        parse_expression(text).unwrap().root.render()
    }

    fn failure(text: &str) -> (Range<usize>, ParseFailure) {
        match parse_expression(text).unwrap_err() {
            Error::Parse { range, failure } => (range, failure),
            error => panic!("expected a parse error, found {error:?}"),
        }
    }

    #[test]
    fn literals_and_names_are_primaries() {
        assert_eq!(render("1"), "1");
        assert_eq!(render("1.5"), "1.5");
        assert_eq!(render("2u8"), "2u8");
        assert_eq!(render("true"), "true");
        assert_eq!(render("false"), "false");
        assert_eq!(render("\"MASK\""), "\"MASK\"");
        assert_eq!(render("albedo"), "`albedo`");
        assert_eq!(render("`foo bar`"), "`foo bar`");
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        assert_eq!(render("a + b * c"), "(+ `a` (* `b` `c`))");
        assert_eq!(render("a * b + c"), "(+ (* `a` `b`) `c`)");
        assert_eq!(render("(a + b) * c"), "(* (+ `a` `b`) `c`)");
    }

    #[test]
    fn arithmetic_associates_left_to_right() {
        assert_eq!(render("a - b - c"), "(- (- `a` `b`) `c`)");
        assert_eq!(render("a / b / c"), "(/ (/ `a` `b`) `c`)");
        assert_eq!(render("a * b / c"), "(/ (* `a` `b`) `c`)");
    }

    #[test]
    fn comparisons_bind_looser_than_arithmetic_and_tighter_than_logic() {
        assert_eq!(
            render("a + 1 > b && c > d"),
            "(&& (> (+ `a` 1) `b`) (> `c` `d`))"
        );
        assert_eq!(render("a == b"), "(== `a` `b`)");
        assert_eq!(render("a != b"), "(!= `a` `b`)");
        assert_eq!(render("a <= b"), "(<= `a` `b`)");
        assert_eq!(render("a >= b"), "(>= `a` `b`)");
        assert_eq!(render("a < b"), "(< `a` `b`)");
    }

    #[test]
    fn a_comparison_chain_parses_for_the_checker_to_reject() {
        assert_eq!(render("a < b < c"), "(< (< `a` `b`) `c`)");
    }

    #[test]
    fn logical_operators_stratify_and_then_xor_then_or() {
        assert_eq!(render("a || b ^ c && d"), "(|| `a` (^ `b` (&& `c` `d`)))");
        assert_eq!(render("a && b && c"), "(&& (&& `a` `b`) `c`)");
        assert_eq!(render("a ^ b ^ c"), "(^ (^ `a` `b`) `c`)");
        assert_eq!(render("a || b || c"), "(|| (|| `a` `b`) `c`)");
    }

    #[test]
    fn unary_operators_nest_and_bind_tighter_than_arithmetic() {
        assert_eq!(render("- -x"), "(- (- `x`))");
        assert_eq!(render("--x"), "(- (- `x`))");
        assert_eq!(render("-a * b"), "(* (- `a`) `b`)");
        assert_eq!(render("!a && b"), "(&& (! `a`) `b`)");
        assert_eq!(render("!!a"), "(! (! `a`))");
        assert_eq!(render("-a.r"), "(- (. `a` r))");
    }

    #[test]
    fn postfixes_chain_left_to_right() {
        assert_eq!(render("color[0].rgb"), "(. ([] `color` 0) rgb)");
        assert_eq!(render("color.rgb[0]"), "([] (. `color` rgb) 0)");
        assert_eq!(render("v.rgba.xy.r"), "(. (. (. `v` rgba) xy) r)");
        assert_eq!(render("a[i][j]"), "([] ([] `a` `i`) `j`)");
    }

    #[test]
    fn literals_and_groups_take_postfixes() {
        assert_eq!(render("0.5.rrr"), "(. 0.5 rrr)");
        assert_eq!(render("2.rr"), "(. 2 rr)");
        assert_eq!(render("(a + b).x"), "(. (+ `a` `b`) x)");
        assert_eq!(render("rgb(1, 2, 3).g"), "(. (rgb 1 2 3) g)");
    }

    #[test]
    fn the_parser_takes_any_member_after_the_dot() {
        assert_eq!(render("v.xg"), "(. `v` xg)");
        assert_eq!(render("v.true"), "(. `v` true)");
        assert_eq!(render("v.swatch"), "(. `v` swatch)");
    }

    #[test]
    fn an_index_takes_any_expression() {
        assert_eq!(render("tint[i + 1]"), "([] `tint` (+ `i` 1))");
        assert_eq!(render("tint[ 0 ]"), "([] `tint` 0)");
    }

    #[test]
    fn calls_take_their_arguments() {
        assert_eq!(render("r(x)"), "(r `x`)");
        assert_eq!(render("rg(x, y)"), "(rg `x` `y`)");
        assert_eq!(render("rgb(o, rough, m)"), "(rgb `o` `rough` `m`)");
        assert_eq!(render("rgba(1, 1, 1, 1)"), "(rgba 1 1 1 1)");
        assert_eq!(render("max(e)"), "(max `e`)");
        assert_eq!(render("max(a, 0.001)"), "(max `a` 0.001)");
        assert_eq!(
            render("clamp(strength / 4, 0, 1)"),
            "(clamp (/ `strength` 4) 0 1)"
        );
        assert_eq!(
            render("mix(\"OPAQUE\", \"MASK\", min(c.a) < 1)"),
            "(mix \"OPAQUE\" \"MASK\" (< (min (. `c` a)) 1))"
        );
        assert_eq!(render("swatchSum(face(1u32))"), "(swatchSum (face 1u32))");
        assert_eq!(
            render("all(color.rgb > 0.9)"),
            "(all (> (. `color` rgb) 0.9))"
        );
    }

    #[test]
    fn default_takes_a_name_then_a_fallback() {
        assert_eq!(
            render("default(occlusionStrength, 1)"),
            "(default `occlusionStrength` 1)"
        );
        assert_eq!(
            render("default(`foo bar`, rgb(0, 0, 0))"),
            "(default `foo bar` (rgb 0 0 0))"
        );
        assert_eq!(
            failure("default(min, 1)"),
            (
                8..11,
                ParseFailure::ReservedName {
                    name: "min".to_owned()
                }
            )
        );
        assert_eq!(
            failure("default(1, 1)"),
            (
                8..9,
                ParseFailure::Unexpected {
                    found: "the number `1`".to_owned(),
                    expected: "a name"
                }
            )
        );
    }

    #[test]
    fn a_wrong_argument_count_errors_over_the_call() {
        assert_eq!(
            failure("rgb(1, 2)"),
            (
                0..9,
                ParseFailure::Arity {
                    function: "rgb",
                    expected: "3".to_owned(),
                    found: 2
                }
            )
        );
        assert_eq!(
            failure("min()"),
            (
                0..5,
                ParseFailure::Arity {
                    function: "min",
                    expected: "1 or 2".to_owned(),
                    found: 0
                }
            )
        );
        assert_eq!(
            failure("abs(a, b)"),
            (
                0..9,
                ParseFailure::Arity {
                    function: "abs",
                    expected: "1".to_owned(),
                    found: 2
                }
            )
        );
    }

    #[test]
    fn a_reserved_name_is_not_a_value() {
        assert_eq!(
            failure("min"),
            (3..3, ParseFailure::UnexpectedEnd { expected: "`(`" })
        );
        assert_eq!(
            failure("min + 1"),
            (
                4..5,
                ParseFailure::Unexpected {
                    found: "`+`".to_owned(),
                    expected: "`(`"
                }
            )
        );
    }

    #[test]
    fn unbalanced_groups_error() {
        assert_eq!(
            failure("(a + b"),
            (6..6, ParseFailure::UnexpectedEnd { expected: "`)`" })
        );
        assert_eq!(
            failure("a + b)"),
            (
                5..6,
                ParseFailure::Unexpected {
                    found: "`)`".to_owned(),
                    expected: "the end of the expression"
                }
            )
        );
        assert_eq!(
            failure("tint[0"),
            (6..6, ParseFailure::UnexpectedEnd { expected: "`]`" })
        );
    }

    #[test]
    fn an_expression_takes_no_terminator() {
        assert_eq!(
            failure("a + b;"),
            (
                5..6,
                ParseFailure::Unexpected {
                    found: "`;`".to_owned(),
                    expected: "the end of the expression"
                }
            )
        );
    }

    #[test]
    fn a_dangling_operator_errors_at_the_end() {
        assert_eq!(
            failure("a +"),
            (
                3..3,
                ParseFailure::UnexpectedEnd {
                    expected: "an expression"
                }
            )
        );
        assert_eq!(
            failure(""),
            (
                0..0,
                ParseFailure::UnexpectedEnd {
                    expected: "an expression"
                }
            )
        );
    }
}
