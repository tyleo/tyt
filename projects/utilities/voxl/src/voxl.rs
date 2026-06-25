use crate::commands::{ToGoxl, ToMvox, ToQbcl, ToVmax, ToVoxj};
use clap::Subcommand;

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Voxl {
    #[command(name = "to-goxl")]
    ToGoxl(ToGoxl),
    #[command(name = "to-mvox")]
    ToMvox(ToMvox),
    #[command(name = "to-qbcl")]
    ToQbcl(ToQbcl),
    #[command(name = "to-vmax")]
    ToVmax(ToVmax),
    #[command(name = "to-voxj")]
    ToVoxj(ToVoxj),
}

impl Voxl {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            Voxl::ToGoxl(to_goxl) => to_goxl.execute(_dependencies),
            Voxl::ToMvox(to_mvox) => to_mvox.execute(_dependencies),
            Voxl::ToQbcl(to_qbcl) => to_qbcl.execute(_dependencies),
            Voxl::ToVmax(to_vmax) => to_vmax.execute(_dependencies),
            Voxl::ToVoxj(to_voxj) => to_voxj.execute(_dependencies),
        }
    }
}
