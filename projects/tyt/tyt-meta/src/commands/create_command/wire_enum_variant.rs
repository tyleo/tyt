use crate::{Dependencies, Result, commands::create_command};
use std::path::Path;

/// Reads a subcommand enum source file, inserts a variant + match arm for
/// `name`/`command`, and writes it back.
pub fn wire_enum_variant(
    deps: &impl Dependencies,
    enum_path: &Path,
    name: &str,
    command: &str,
) -> Result<()> {
    let contents = deps.read_to_string(enum_path)?;
    let updated = create_command::insert_enum_variant(&contents, name, command)?;
    deps.write(enum_path, &updated)
}
