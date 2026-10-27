use crate::{Error, ParseFailure, Token};

/// The error for a token where the parser expected something else.
pub fn unexpected(token: &Token, expected: &'static str) -> Error {
    Error::Parse {
        range: token.range.clone(),
        failure: ParseFailure::Unexpected {
            found: token.kind.describe(),
            expected,
        },
    }
}
