use crate::{Dependencies, Result, commands::create_command};
use clap::{ArgAction, Parser};

/// Scaffolds a new sub-crate or adds a command to an existing one.
///
/// Without `--parent`, creates a brand-new sub-crate with all boilerplate and
/// adds it to the workspace. By default the crate is named `tyt-{command}`,
/// placed at `projects/tyt/tyt-{command}`, and wired into the top-level `tyt`
/// binary. Pass `--dir` to place it under a different `projects/` group and
/// `--prefix false` to drop the `tyt-` name prefix, which produces a standalone
/// crate that is added to the workspace but not the `tyt` binary.
///
/// With `--parent`, adds a command to an existing sub-crate. The first `--parent`
/// is the crate suffix; repeat it to nest the command under parent command groups,
/// which are created on demand (e.g. `--parent voxj --parent from` adds a command
/// at `tyt voxj from <command>`). Here `--dir` and `--prefix` are inferred from the
/// existing crate when omitted, and only needed to disambiguate same-named crates.
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

    /// Group directory under `projects/` to place the crate in (e.g. `utilities`).
    /// Defaults to `tyt` when creating; inferred from disk when adding with
    /// `--parent`.
    #[arg(value_name = "dir", long)]
    pub dir: Option<String>,

    /// Whether to apply the `tyt-` crate and `Tyt` enum name prefix. Set
    /// `--prefix false` for a standalone crate not wired into the `tyt` binary.
    /// Defaults to true when creating; inferred from disk when adding with
    /// `--parent`.
    #[arg(value_name = "prefix", long, action = ArgAction::Set)]
    pub prefix: Option<bool>,
}

impl CreateCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self.parent.as_slice() {
            [] => create_command::create_crate(&self, &dependencies),
            parents => create_command::add_command_to_crate(&self, &dependencies, parents),
        }
    }
}
