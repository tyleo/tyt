use crate::commands::HierarchyShow;
use clap::Subcommand;

/// The `hierarchy` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum HierarchyCommand {
    #[command(name = "show")]
    HierarchyShow(HierarchyShow),
}

impl HierarchyCommand {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            HierarchyCommand::HierarchyShow(show) => show.execute(_dependencies),
        }
    }
}
