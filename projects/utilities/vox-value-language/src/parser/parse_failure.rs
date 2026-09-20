use std::fmt::{Display, Formatter, Result as FmtResult};

/// The token or grammar rule a parse broke.
#[derive(Clone, Debug, PartialEq)]
pub enum ParseFailure {
    /// A function took the wrong number of arguments.
    Arity {
        function: &'static str,
        expected: String,
        found: usize,
    },

    /// A swizzle dot or an index bracket sat apart from its source.
    DetachedPostfix,

    /// A backtick-quoted name held nothing.
    EmptyName,

    /// A statement did not bind a name.
    ExpectedBinding,

    /// A swizzle dot was not followed by its member.
    MemberExpected,

    /// A reserved name stood where a name was expected.
    ReservedName { name: String },

    /// A type suffix followed a number with a decimal point.
    SuffixOnFraction { suffix: String },

    /// A token stood where the grammar expected another.
    Unexpected {
        found: String,
        expected: &'static str,
    },

    /// A character belongs to no token.
    UnexpectedCharacter { character: char },

    /// The text ended where the grammar expected more.
    UnexpectedEnd { expected: &'static str },

    /// A number carried a suffix that names no type.
    UnknownSuffix { suffix: String },

    /// A backtick-quoted name never closed.
    UnterminatedName,

    /// A string literal never closed.
    UnterminatedString,
}

impl Display for ParseFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            ParseFailure::Arity {
                function,
                expected,
                found,
            } => write!(
                formatter,
                "`{function}` takes {expected} argument(s), found {found}"
            ),

            ParseFailure::DetachedPostfix => write!(
                formatter,
                "a swizzle dot or index bracket must attach to its source"
            ),

            ParseFailure::EmptyName => write!(formatter, "a backtick-quoted name is empty"),

            ParseFailure::ExpectedBinding => {
                write!(formatter, "a statement binds a name: `name = expr;`")
            }

            ParseFailure::MemberExpected => write!(formatter, "a swizzle dot needs its member"),

            ParseFailure::ReservedName { name } => {
                write!(
                    formatter,
                    "`{name}` is reserved; backtick-quote it as a name"
                )
            }

            ParseFailure::SuffixOnFraction { suffix } => write!(
                formatter,
                "the suffix `{suffix}` belongs to a whole number, not one with a decimal point"
            ),

            ParseFailure::Unexpected { found, expected } => {
                write!(formatter, "expected {expected}, found {found}")
            }

            ParseFailure::UnexpectedCharacter { character } => {
                write!(formatter, "unexpected character `{character}`")
            }

            ParseFailure::UnexpectedEnd { expected } => {
                write!(formatter, "expected {expected}, found the end of the text")
            }

            ParseFailure::UnknownSuffix { suffix } => {
                write!(formatter, "`{suffix}` is not a number suffix")
            }

            ParseFailure::UnterminatedName => {
                write!(formatter, "a backtick-quoted name never closes")
            }

            ParseFailure::UnterminatedString => write!(formatter, "a string literal never closes"),
        }
    }
}
