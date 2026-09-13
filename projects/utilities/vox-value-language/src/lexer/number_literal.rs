use crate::lexer::NumberSuffix;

/// A number token: its digits, whether a decimal point made it an `f32`,
/// and the suffix that pinned a type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NumberLiteral {
    /// The digits and the decimal point, without any suffix.
    pub(crate) text: String,

    /// Whether the text holds a decimal point.
    pub(crate) fraction: bool,

    /// The suffix, when the literal carries one.
    pub(crate) suffix: Option<NumberSuffix>,
}
