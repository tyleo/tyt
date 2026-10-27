use crate::{
    Dependencies, Result,
    commands::{ProfileMeshDoc, ProfileObject, ProfilePalette, ProfileSdfDoc},
};
use clap::Subcommand;

/// The `profile` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileCommand {
    #[command(name = "mesh-doc")]
    ProfileMeshDoc(ProfileMeshDoc),

    #[command(name = "object")]
    ProfileObject(ProfileObject),

    #[command(name = "palette")]
    ProfilePalette(ProfilePalette),

    #[command(name = "sdf-doc")]
    ProfileSdfDoc(ProfileSdfDoc),
}

impl ProfileCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileCommand::ProfileMeshDoc(mesh_doc) => mesh_doc.execute(dependencies),
            ProfileCommand::ProfileObject(object) => object.execute(dependencies),
            ProfileCommand::ProfilePalette(palette) => palette.execute(dependencies),
            ProfileCommand::ProfileSdfDoc(sdf_doc) => sdf_doc.execute(dependencies),
        }
    }
}
