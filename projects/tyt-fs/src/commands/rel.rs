use crate::{
    Dependencies, Error, Result,
    utilities::{self, RelativeTo},
};
use clap::Parser;
use std::path::PathBuf;

/// Constructs a path relative to a saved base path in `.tytconfig` or
/// `.tytusrconfig`.
///
/// Base paths are read from the merged `fs.rel` maps of `<git-root>/.tytconfig`
/// and `<git-root>/.tytusrconfig`; on a duplicate key the `.tytusrconfig` value
/// wins. Each base path is resolved relative to the git root, then
/// `relative-path` is joined onto it and the result re-expressed per
/// `--relative-to`.
#[derive(Clone, Debug, Parser)]
#[command(name = "rel")]
pub struct Rel {
    /// The `fs.rel` key naming the base path to build from.
    #[arg(value_name = "base-name")]
    base_name: String,

    /// The path appended to the resolved base path.
    #[arg(value_name = "relative-path")]
    relative_path: PathBuf,

    /// What the output path is expressed relative to.
    #[arg(
        value_name = "relative-to",
        long,
        value_enum,
        default_value_t = RelativeTo::Cwd,
    )]
    relative_to: RelativeTo,
}

impl Rel {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let config = dependencies.rel_config()?.ok_or(Error::ConfigNotFound)?;

        let base = config
            .bases
            .get(&self.base_name)
            .ok_or_else(|| Error::RelBaseNotFound(self.base_name.clone()))?;

        let joined = base.join(&self.relative_path);

        let reference = match self.relative_to {
            RelativeTo::Cwd => dependencies.current_dir()?,
            RelativeTo::Config => config.config_dir.clone(),
        };

        let output = utilities::relativize(&reference, &joined);
        dependencies.write_stdout(output.as_os_str().as_encoded_bytes())?;
        dependencies.write_stdout(b"\n")?;
        Ok(())
    }
}
