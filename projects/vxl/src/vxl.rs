use crate::{
    Dependencies, Result,
    commands::{MeshDoc, Node, Object, Palette, Profile, SdfDoc, VoxDoc},
};
use clap::Subcommand;

/// Works with voxels.
// The command line parses once, so the variants' sizes never matter.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Vxl {
    #[command(name = "mesh-doc")]
    MeshDoc(MeshDoc),

    #[command(name = "node")]
    Node(Node),

    #[command(name = "object")]
    Object(Object),

    #[command(name = "palette")]
    Palette(Palette),

    #[command(name = "profile")]
    Profile(Profile),

    #[command(name = "sdf-doc")]
    SdfDoc(SdfDoc),

    #[command(name = "vox-doc")]
    VoxDoc(VoxDoc),
}

impl Vxl {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            Vxl::MeshDoc(mesh_doc) => mesh_doc.execute(dependencies),
            Vxl::Node(node) => node.execute(dependencies),
            Vxl::Object(object) => object.execute(dependencies),
            Vxl::Palette(palette) => palette.execute(dependencies),
            Vxl::Profile(profile) => profile.execute(dependencies),
            Vxl::SdfDoc(sdf_doc) => sdf_doc.execute(dependencies),
            Vxl::VoxDoc(vox_doc) => vox_doc.execute(dependencies),
        }
    }
}
