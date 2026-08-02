use clap::ValueEnum;

/// How `palette show`'s text layouts label each collection.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowLabel {
    /// No labels.
    #[value(name = "none")]
    None,

    /// Full dot-joined paths, like `0."baseColor".a`.
    #[value(name = "concat")]
    Concat,

    /// Nested markdown headings over collections labeled by their leaf
    /// segment alone.
    #[value(name = "header")]
    Header,
}
