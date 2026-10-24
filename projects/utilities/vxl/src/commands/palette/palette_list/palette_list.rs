use crate::{Dependencies, Result, VoxelInput, cli_value_parser, load_vox_main, parse_id_selector};
use clap::{ArgAction, Parser};
use voxcore::{BVoxPalette, VoxMain};
use voxsmith::{
    operations::palette::{PaletteListFields, PaletteListLayout, palette_list},
    utilities::IdSelector,
};

/// Lists every palette in a document, one row apiece.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct PaletteList {
    #[command(flatten)]
    input: VoxelInput,

    /// Palette ids: an integer, an inclusive `a-b` range, or `*` for every
    /// palette, unioned. With none given, every palette is listed.
    #[arg(value_name = "palette", value_parser = parse_id_selector::<BVoxPalette>)]
    filters: Vec<IdSelector<BVoxPalette>>,

    /// How to lay out the listing.
    #[arg(
        value_name = "layout",
        long,
        default_value = "box-hierarchy",
        value_parser = cli_value_parser::<PaletteListLayout>()
    )]
    layout: PaletteListLayout,

    /// Shows each palette's property keys. `--show-properties false` omits them.
    #[arg(
        value_name = "show-properties",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    show_properties: bool,

    /// Shows each palette's material count. `--show-materials false` omits it.
    #[arg(
        value_name = "show-materials",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    show_materials: bool,

    /// Shows the objects that reference each palette. `--show-objects false`
    /// omits them.
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
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let fields = PaletteListFields {
            properties: self.show_properties,
            materials: self.show_materials,
            objects: self.show_objects,
        };

        let from = self.input.resolve_format()?;

        let main: VoxMain = load_vox_main(&dependencies, from, &self.input.path)?;

        let output = palette_list(&main, &self.filters, fields, self.layout)?;

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }
}
