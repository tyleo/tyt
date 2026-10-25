/// A font a `text` shape reads: an entry of
/// [`SdfState::fonts`](crate::SdfState::fonts).
#[derive(Clone, Debug, PartialEq)]
pub enum SdfFont {
    /// A font file at `path`, relative to the document's directory. A font
    /// collection gives its first face.
    File { name: String, path: String },

    /// The installed face whose family or full name matches `font`, ignoring
    /// case. A family picks its regular face.
    Installed { name: String, font: String },
}

impl SdfFont {
    /// The name that identifies the font in errors.
    pub fn name(&self) -> &str {
        match self {
            SdfFont::File { name, .. } | SdfFont::Installed { name, .. } => name,
        }
    }
}
