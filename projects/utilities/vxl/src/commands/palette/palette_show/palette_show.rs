use crate::{
    Dependencies, Error, Format, Result, Width,
    commands::{PaletteShowLayout, PropertySelector},
};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Prints one or more palette value collections.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct PaletteShow {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// A repeatable selector naming a value collection, three fields:
    /// `<palette> <property> <format>`. The palette is an index or `*`, the
    /// property a key with an optional `.r`/`.g`/`.b`/`.a` color component or
    /// `*`, and the format one of `auto`, `swatch`, `value`, `swatch-value`.
    /// Defaults to `'*' '*' auto` when omitted.
    #[arg(
        value_names = ["palette", "property", "format"],
        long = "property",
        num_args = 3,
        action = clap::ArgAction::Append,
    )]
    property: Vec<String>,

    /// How to arrange the collections, and the serialization to emit.
    #[arg(value_name = "layout", long, default_value = "row")]
    layout: PaletteShowLayout,

    /// Width the `row` layouts wrap to: `terminal` (default), `unlimited`, or a
    /// column count.
    #[arg(value_name = "width", long, default_value = "terminal")]
    width: Width,
}

impl PaletteShow {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        // clap fixes each occurrence at three values, so the flattened list
        // chunks cleanly into one selector per occurrence.
        let selectors = if self.property.is_empty() {
            vec![PropertySelector::default_all_auto()]
        } else {
            self.property
                .chunks(3)
                .map(|chunk| PropertySelector::parse(&chunk[0], &chunk[1], &chunk[2]))
                .collect::<std::result::Result<Vec<_>, String>>()
                .map_err(|message| Error::IO(IOError::new(ErrorKind::InvalidInput, message)))?
        };
        dependencies.palette_show(&self.input, self.from, &selectors, self.layout, self.width)
    }
}
