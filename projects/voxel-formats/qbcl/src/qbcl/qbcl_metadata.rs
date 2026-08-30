/// The seven free-text metadata strings in a `.qbcl` file's header, in the order
/// the Qubicle editor's UI lists them.
///
/// Each is stored on disk as a length-prefixed string and is empty by default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QbclMetadata {
    /// Document title.
    pub title: String,

    /// Document description.
    pub description: String,

    /// Tags.
    pub tags: String,

    /// Author.
    pub author: String,

    /// Company.
    pub company: String,

    /// Website.
    pub website: String,

    /// Copyright notice.
    pub copyright: String,
}
