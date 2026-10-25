use crate::SdfGlyph;
use std::collections::BTreeMap;

/// The decoded glyphs of a font in [`SdfState::fonts`](crate::SdfState::fonts)
/// for the characters its `text` shapes hold, in font units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfFontData {
    /// The units in the font's em.
    pub units_per_em: f64,

    /// The height of a capital letter, or `None` when the font gives none and
    /// has no `H` to measure.
    pub cap_height: Option<f64>,

    /// The glyph of each character the font holds. A character the font lacks
    /// has no entry.
    pub glyphs: BTreeMap<char, SdfGlyph>,
}
