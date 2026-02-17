use crate::{
    Dependencies, Result,
    commands::create_command::{add_command_to_crate, create_crate},
};
use clap::Parser;

/// Scaffolds a new tyt sub-crate or adds a command to an existing one.
///
/// Without `--parent`, creates a brand-new `tyt-{command}` sub-crate with all
/// boilerplate and wires it into the workspace and top-level `tyt` binary.
///
/// With `--parent`, adds a command to an existing sub-crate.
#[derive(Clone, Debug, Parser)]
#[command(name = "create-command")]
pub struct CreateCommand {
    /// PascalCase type name (e.g., `FooBar`).
    #[arg(value_name = "name")]
    pub name: String,

    /// kebab-case CLI name (e.g., `foo-bar`).
    #[arg(value_name = "command")]
    pub command: String,

    /// Description for doc comments, Cargo.toml, and README.
    #[arg(value_name = "description")]
    pub description: String,

    /// Existing crate suffix to add the command to (e.g., `fbx` for `tyt-fbx`).
    #[arg(value_name = "parent", short, long)]
    pub parent: Option<String>,
}

impl CreateCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match &self.parent {
            Some(parent) => add_command_to_crate(&self, &dependencies, parent),
            None => create_crate(&self, &dependencies),
        }
    }
}
