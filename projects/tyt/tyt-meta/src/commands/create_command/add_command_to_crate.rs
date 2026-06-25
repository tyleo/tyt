use crate::{
    Dependencies, Error, Result,
    commands::{CreateCommand, create_command},
};

/// Adds a command to an existing crate, nesting it under zero or more parent
/// command groups.
///
/// `parents[0]` is the crate suffix (e.g. `voxj` for `tyt-voxj`); any remaining
/// entries are parent command groups, created or extended as needed so the new
/// command lands that many levels deep (e.g. `tyt voxj from vmax`).
pub fn add_command_to_crate(
    cmd: &CreateCommand,
    deps: &impl Dependencies,
    parents: &[String],
) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let command_snake = create_command::kebab_to_snake_case(command);

    let (crate_suffix, groups) = parents
        .split_first()
        .ok_or_else(|| Error::Meta("at least one parent is required".to_string()))?;
    let crate_snake = create_command::kebab_to_snake_case(crate_suffix);

    let root = deps.workspace_root()?;
    let crate_dir = root
        .join(create_command::TYT_PROJECT_DIR)
        .join(format!("tyt-{crate_suffix}"));
    if !crate_dir.exists() {
        return Err(Error::Meta(format!(
            "parent crate not found: {}",
            crate_dir.display()
        )));
    }

    let commands_dir = crate_dir.join("src/commands");
    let mod_path = commands_dir.join("mod.rs");

    // Walk the parent groups from the crate root inward, ensuring each exists, and
    // track the enum the new command will be wired into (the crate root enum when
    // there are no groups, otherwise the innermost group's subcommand enum).
    let mut enum_path = crate_dir.join(format!("src/tyt_{crate_snake}.rs"));
    for group in groups {
        enum_path =
            create_command::ensure_group(deps, &commands_dir, &mod_path, &enum_path, group)?;
    }

    // 1. Create the leaf command file.
    let cmd_file = commands_dir.join(format!("{command_snake}.rs"));
    if cmd_file.exists() {
        return Err(Error::Meta(format!(
            "command file already exists: {}",
            cmd_file.display()
        )));
    }
    deps.write(
        &cmd_file,
        &create_command::command_file_template(name, command, description),
    )?;

    // 2. Register the module and wire the variant into the target enum.
    create_command::register_command_mod(deps, &mod_path, &command_snake)?;
    create_command::wire_enum_variant(deps, &enum_path, name, command)?;

    let path = parents.join(" ");
    deps.write_stdout(
        format!("Added `{name}` command. Run it with `tyt {path} {command}`.\n").as_bytes(),
    )?;

    Ok(())
}
