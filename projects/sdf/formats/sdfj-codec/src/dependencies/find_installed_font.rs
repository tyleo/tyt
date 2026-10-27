use crate::SdfjFontFile;

/// Finds the installed faces a document's `installed` fonts ask for.
pub trait FindInstalledFont {
    /// The installed face whose family or full name matches `font`, ignoring
    /// case, or `None` when no face matches. A family picks its regular face.
    fn find_installed_font(&self, font: &str) -> Result<Option<SdfjFontFile>, String>;
}
