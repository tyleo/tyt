use crate::{
    Dependencies, Error, Result, VoxelInput, Width, cli_value_parser,
    commands::parse_property_selector,
};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    num::NonZeroU8,
};
use voxconv::load;
use voxcore::VoxMain;
use voxsmith::operations::palette_show::{
    PaletteShowLabel, PaletteShowLayout, PaletteShowOptions, PaletteShowTableShape,
    PropertySelector, palette_show,
};

/// Prints one or more palette value collections.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct PaletteShow {
    #[command(flatten)]
    input: VoxelInput,

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

        let from = self.input.resolve_format()?;

        let main: VoxMain = load(&dependencies, from, &self.input.path)?;

        let options = PaletteShowOptions {
            layout: self.layout,
            label: self.label,
            header_level: self.header_level,
            table_shape: self.table_shape,
            width: resolve_width(&dependencies, self.width),
        };

        let output = palette_show(&main, &selectors, &options)?;

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }
}

/// The column budget a `Width` resolves to, or `None` for no wrapping. A
/// `Terminal` width with no terminal on stdout, as when the output is piped,
/// also resolves to no wrapping.
fn resolve_width<D: Dependencies>(dependencies: &D, width: Width) -> Option<usize> {
    match width {
        Width::Unlimited => None,
        Width::Columns(columns) => Some(columns),
        Width::Terminal => dependencies.terminal_columns(),
    }
}
