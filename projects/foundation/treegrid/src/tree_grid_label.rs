/// One path segment of a node's label.
///
/// The variants name the rendering effect in the text layouts, not the
/// provenance: `Bare` prints as-is, for trusted identifiers like
/// `transform` or `0`; `Quoted` prints `{:?}`-escaped, for
/// user-entered strings like `"baseColorFactor"`. The JSON layouts
/// always emit the raw string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeGridLabel {
    /// Prints as-is in the text layouts.
    Bare(String),

    /// Prints `{:?}`-escaped in the text layouts.
    Quoted(String),
}

impl TreeGridLabel {
    /// Creates a label that prints as-is.
    pub fn bare(text: impl Into<String>) -> Self {
        Self::Bare(text.into())
    }

    /// Creates a label that prints `{:?}`-escaped.
    pub fn quoted(text: impl Into<String>) -> Self {
        Self::Quoted(text.into())
    }

    /// The raw segment text, as the JSON layouts emit it.
    pub fn text(&self) -> &str {
        match self {
            Self::Bare(text) | Self::Quoted(text) => text,
        }
    }

    /// The segment as the text layouts print it: the raw text for
    /// `Bare`, `{:?}`-quoted for `Quoted`.
    pub fn render(&self) -> String {
        match self {
            Self::Bare(text) => text.clone(),
            Self::Quoted(text) => format!("{text:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TreeGridLabel;

    #[test]
    fn bare_renders_as_is() {
        assert_eq!(TreeGridLabel::bare("transform").render(), "transform");
    }

    #[test]
    fn quoted_renders_quote_wrapped() {
        assert_eq!(
            TreeGridLabel::quoted("baseColorFactor").render(),
            "\"baseColorFactor\""
        );
    }

    #[test]
    fn quoted_escapes_inner_quotes() {
        assert_eq!(
            TreeGridLabel::quoted("has \"quotes\"").render(),
            r#""has \"quotes\"""#
        );
    }

    #[test]
    fn quoted_renders_the_empty_string() {
        assert_eq!(TreeGridLabel::quoted("").render(), "\"\"");
    }

    #[test]
    fn text_returns_the_raw_segment() {
        assert_eq!(
            TreeGridLabel::quoted("has \"quotes\"").text(),
            "has \"quotes\""
        );
    }
}
