use crate::lexer::TokenKind;
use std::ops::Range;

/// A token and the byte range it came from.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Token {
    /// What the token is.
    pub(crate) kind: TokenKind,

    /// The byte range in the text.
    pub(crate) range: Range<usize>,
}
