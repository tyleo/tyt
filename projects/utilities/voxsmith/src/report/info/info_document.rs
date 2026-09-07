/// What [`render_info`](crate::render_info) reports about a document beyond
/// its scene: where it came from and what the source carried that the loaded
/// state drops.
#[derive(Clone, Copy, Debug)]
pub struct InfoDocument<'a> {
    /// The file name, printed as the report title.
    pub name: &'a str,

    /// The short name of the format the document was read as.
    pub format: &'a str,

    /// The source's format version, for a format that records one.
    pub format_version: Option<u32>,

    /// Whether the source carried an `ext` block.
    pub has_ext: bool,
}
