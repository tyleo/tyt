use crate::{Dependencies, Result, commands::create_command};
use std::path::Path;

/// Reads a `commands/mod.rs`, inserts a `mod`/`pub use` entry for `module`, and
/// writes it back.
pub fn register_command_mod(deps: &impl Dependencies, mod_path: &Path, module: &str) -> Result<()> {
    let contents = deps.read_to_string(mod_path)?;
    let updated = create_command::insert_command_mod(&contents, module);
    deps.write(mod_path, &updated)
}
