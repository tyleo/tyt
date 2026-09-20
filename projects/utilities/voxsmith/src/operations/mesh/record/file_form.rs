/// A file's form; entries of one JSON file merge by path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileForm {
    /// A JSON entry under its name.
    Json {
        /// The entry's key.
        name: String,
    },

    /// An 8-bit PNG.
    Png,
}
