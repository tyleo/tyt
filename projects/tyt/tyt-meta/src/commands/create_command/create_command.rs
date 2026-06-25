use crate::{Dependencies, Result, commands::create_command};
use clap::Parser;

/// Scaffolds a new tyt sub-crate or adds a command to an existing one.
///
/// Without `--parent`, creates a brand-new `tyt-{command}` sub-crate with all
/// boilerplate and wires it into the workspace and top-level `tyt` binary.
///
/// With `--parent`, adds a command to an existing sub-crate. The first `--parent`
/// is the crate suffix; repeat it to nest the command under parent command groups,
/// which are created on demand (e.g. `--parent voxj --parent from` adds a command
/// at `tyt voxj from <command>`).
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

    /// Parent command path, from crate suffix inward (e.g. `fbx` for `tyt-fbx`).
    /// Repeat to nest under command groups (e.g. `-p voxj -p from`).
    #[arg(value_name = "parent", short, long)]
    pub parent: Vec<String>,
}

impl CreateCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self.parent.as_slice() {
            [] => create_command::create_crate(&self, &dependencies),
            parents => create_command::add_command_to_crate(&self, &dependencies, parents),
        }
    }
}
