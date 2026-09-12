use crate::{Dependencies, Result, VoxelInput, cli_value_parser, parse_index_range};
use clap::{ArgAction, Parser};
use voxconv::load;
use voxcore::VoxMain;
use voxsmith::{
    operations::palette_list::{PaletteListFields, PaletteListLayout, palette_list},
    utilities::IndexRange,
};

/// Lists every palette in a document, one row apiece.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct PaletteList {
    #[command(flatten)]
    input: VoxelInput,

    /// Palette-index filters, each a single index `1` or an inclusive range
    /// `1-5`, unioned over all values. Given none, every palette is listed.
    #[arg(value_name = "filter", value_parser = parse_index_range)]
    filters: Vec<IndexRange>,

    /// How to lay out the listing.
    #[arg(
        value_name = "layout",
        long,
        default_value = "hierarchy",
        value_parser = cli_value_parser::<PaletteListLayout>()
    )]
    layout: PaletteListLayout,

    /// Show each palette's property keys. `--show-properties false` drops them.
    #[arg(
        value_name = "show-properties",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    show_properties: bool,

    /// Show each palette's material count. `--show-materials false` drops it.
    #[arg(
        value_name = "show-materials",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    show_materials: bool,

    /// Show the objects that reference each palette. `--show-objects false`
    /// drops them.
    #[arg(
        value_name = "show-objects",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    show_objects: bool,
}

impl PaletteList {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let fields = PaletteListFields {
            properties: self.show_properties,
            materials: self.show_materials,
            objects: self.show_objects,
        };

        let from = self.input.resolve_format()?;

        let main: VoxMain = load(&dependencies, from, &self.input.path)?;

        let output = palette_list(&main, &self.filters, fields, self.layout)?;

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }
}
