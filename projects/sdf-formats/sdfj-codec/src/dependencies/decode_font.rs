use crate::{SdfjFontData, SdfjFontFile};
use std::collections::BTreeSet;

/// Decodes the glyphs of a font file.
pub trait DecodeFont {
    /// The glyphs of `characters` in `file`'s face, or the reason the face
    /// does not decode. A font without a cap height takes the top of its `H`.
    fn decode_font(
        &self,
        file: &SdfjFontFile,
        characters: &BTreeSet<char>,
    ) -> Result<SdfjFontData, String>;
}
