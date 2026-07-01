use crate::commands::{Hierarchy, Info, Mesh, Palette, Validate, Voxelize};
use crate::{Dependencies, Result, commands::To};
use clap::Subcommand;

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Vxl {
    #[command(name = "hierarchy")]
    Hierarchy(Hierarchy),
    #[command(name = "info")]
    Info(Info),
    #[command(name = "mesh")]
    Mesh(Mesh),
    #[command(name = "palette")]
    Palette(Palette),
    #[command(name = "to")]
    To(To),
    #[command(name = "validate")]
    Validate(Validate),
    #[command(name = "voxelize")]
    Voxelize(Voxelize),
}

impl Vxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            Vxl::Hierarchy(hierarchy) => hierarchy.execute(dependencies),
            Vxl::Info(info) => info.execute(dependencies),
            Vxl::Mesh(mesh) => mesh.execute(dependencies),
            Vxl::Palette(palette) => palette.execute(dependencies),
            Vxl::To(to) => to.execute(dependencies),
            Vxl::Validate(validate) => validate.execute(dependencies),
            Vxl::Voxelize(voxelize) => voxelize.execute(dependencies),
        }
    }
}
