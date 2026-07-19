//! Box-drawing glyphs for the `hierarchy` layout's connectors and prefix
//! extensions.

/// Box drawings up and right: the connector before a last child.
pub(crate) const CONNECTOR_LAST: char = '\u{2514}';

/// Box drawings vertical and right: the connector before a non-last child.
pub(crate) const CONNECTOR_MID: char = '\u{251C}';

/// The prefix extension under a last child: two spaces.
pub(crate) const EXTENSION_LAST: &str = "  ";

/// The prefix extension under a non-last child: box drawings vertical, a space.
pub(crate) const EXTENSION_MID: &str = "\u{2502} ";
