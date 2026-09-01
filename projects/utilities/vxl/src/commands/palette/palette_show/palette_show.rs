use crate::{
    Dependencies, Error, Format, Result, Width, cli_value_parser, commands::parse_property_selector,
};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    num::NonZeroU8,
    path::PathBuf,
};
use voxsmith::{PaletteShowLabel, PaletteShowLayout, PaletteShowTableShape, PropertySelector};

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

    /// A repeatable selector naming a value collection, four fields:
    /// `<palette> <property> <presentation> <reading>`. The palette is an
    /// index or `*`; the property a key with an optional `.r`/`.g`/`.b`/`.a`
    /// or `.x`/`.y`/`.z`/`.w` component, or `*`; the presentation one of
    /// `auto`, `swatch`, `swatch-value`, `value`; the reading one of `auto`,
    /// `linear-float`, `plain`, `srgb-float`, `srgb-hex`. Defaults to
    /// `'*' '*' auto auto` when omitted.
    #[arg(
        value_names = ["palette", "property", "presentation", "reading"],
        long = "property",
        num_args = 4,
        action = clap::ArgAction::Append,
    )]
    property: Vec<String>,

    /// How to arrange the value collections, and the serialization to emit.
    #[arg(
        value_name = "layout",
        long,
        default_value = "rows",
        value_parser = cli_value_parser::<PaletteShowLayout>()
    )]
    layout: PaletteShowLayout,

    /// How the text layouts label each value collection. Defaults to `concat`,
    /// full dot-joined paths.
    #[arg(
        value_name = "label",
        long,
        value_parser = cli_value_parser::<PaletteShowLabel>()
    )]
    label: Option<PaletteShowLabel>,

    /// The markdown level of the shallowest heading a heading-emitting
    /// render prints. Headings start at `#` when omitted.
    #[arg(value_name = "header-level", long)]
    header_level: Option<NonZeroU8>,

    /// How the `tables` layout shapes its tables. Defaults to `nested`,
    /// one table per palette group under headings.
    #[arg(
        value_name = "table-shape",
        long,
        value_parser = cli_value_parser::<PaletteShowTableShape>()
    )]
    table_shape: Option<PaletteShowTableShape>,

    /// Width the `rows` layout wraps to: `terminal` (default), `unlimited`,
    /// or a column count.
    #[arg(value_name = "width", long, default_value = "terminal")]
    width: Width,
}

impl PaletteShow {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        // clap fixes each occurrence at four values, so the flattened list
        // chunks cleanly into one selector per occurrence.
        let selectors = if self.property.is_empty() {
            vec![PropertySelector::default()]
        } else {
            self.property
                .chunks(4)
                .map(|chunk| parse_property_selector(&chunk[0], &chunk[1], &chunk[2], &chunk[3]))
                .collect::<std::result::Result<Vec<_>, String>>()
                .map_err(|message| Error::IO(IOError::new(ErrorKind::InvalidInput, message)))?
        };

        dependencies.palette_show(
            &self.input,
            self.from,
            &selectors,
            self.layout,
            self.label,
            self.header_level,
            self.table_shape,
            self.width,
        )
    }
}
