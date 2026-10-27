use crate::{
    Dependencies, Result, cli_value_parser,
    commands::{list_profiles, load_mesh_doc_voxelize_profile_set},
};
use clap::{ArgAction, Parser};
use voxsmith::operations::profile::ProfileListLayout;

/// Lists the profiles `mesh-doc voxelize --profile` can apply, grouped by the
/// `.vxlconfig` that supplies each.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct ProfileMeshDocVoxelizeList {
    /// How to lay out the listing.
    #[arg(
        value_name = "layout",
        long,
        default_value = "box-hierarchy",
        value_parser = cli_value_parser::<ProfileListLayout>()
    )]
    layout: ProfileListLayout,

    /// Shows each profile's description. `--show-descriptions false` omits
    /// the descriptions.
    #[arg(
        value_name = "show-descriptions",
        long,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    show_descriptions: bool,
}

impl ProfileMeshDocVoxelizeList {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profiles = load_mesh_doc_voxelize_profile_set(&dependencies)?;
        Ok(dependencies.write_stdout(
            list_profiles(&profiles, self.layout, self.show_descriptions).as_bytes(),
        )?)
    }
}
