use crate::{
    Dependencies, Result,
    commands::{ObjectSetEditBounds, ObjectSetName, ObjectSetOrigin},
};
use clap::Subcommand;

/// The `object set` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ObjectSetCommand {
    #[command(name = "edit-bounds")]
    ObjectSetEditBounds(ObjectSetEditBounds),

    #[command(name = "name")]
    ObjectSetName(ObjectSetName),

    #[command(name = "origin")]
    ObjectSetOrigin(ObjectSetOrigin),
}

impl ObjectSetCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ObjectSetCommand::ObjectSetEditBounds(edit_bounds) => edit_bounds.execute(dependencies),
            ObjectSetCommand::ObjectSetName(name) => name.execute(dependencies),
            ObjectSetCommand::ObjectSetOrigin(origin) => origin.execute(dependencies),
        }
    }
}
