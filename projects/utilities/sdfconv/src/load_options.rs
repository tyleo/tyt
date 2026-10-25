/// What [`load()`](crate::load()) loads beside the document.
#[derive(Clone, Copy, Debug, Default)]
pub struct LoadOptions {
    /// Loads the document's fonts. A file font's path resolves against the
    /// document's directory.
    pub fonts: bool,
}
