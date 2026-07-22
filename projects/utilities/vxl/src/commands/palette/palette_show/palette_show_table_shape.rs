use clap::ValueEnum;

/// How the `tables` layout shapes its tables.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum PaletteShowTableShape {
    /// One table per palette group, under nested headings.
    #[value(name = "nested")]
    Nested,

    /// One table over every collection, the cross-palette comparison view.
    #[value(name = "flat")]
    Flat,
}
