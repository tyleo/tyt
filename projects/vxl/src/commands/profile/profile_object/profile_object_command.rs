use crate::{
    Dependencies, Result,
    commands::{ProfileObjectMesh, ProfileObjectRender, ProfileObjectVoxels},
};
use clap::Subcommand;

/// The `profile object` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileObjectCommand {
    #[command(name = "mesh")]
    ProfileObjectMesh(ProfileObjectMesh),

    #[command(name = "render")]
    ProfileObjectRender(ProfileObjectRender),

    #[command(name = "voxels")]
    ProfileObjectVoxels(ProfileObjectVoxels),
}

impl ProfileObjectCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileObjectCommand::ProfileObjectMesh(mesh) => mesh.execute(dependencies),
            ProfileObjectCommand::ProfileObjectRender(render) => render.execute(dependencies),
            ProfileObjectCommand::ProfileObjectVoxels(voxels) => voxels.execute(dependencies),
        }
    }
}
