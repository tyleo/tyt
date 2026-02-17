use crate::{
    Dependencies, Error, Result,
    commands::{CreateCommand, create_command},
};

pub fn add_command_to_crate(
    cmd: &CreateCommand,
    deps: &impl Dependencies,
    parent: &str,
) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let command_snake = create_command::kebab_to_snake_case(command);
    let parent_snake = create_command::kebab_to_snake_case(parent);
    let root = deps.workspace_root()?;
    let parent_dir = root.join(format!("projects/tyt-{parent}"));

    if !parent_dir.exists() {
        return Err(Error::Meta(format!(
            "parent crate not found: {}",
            parent_dir.display()
        )));
    }

    // 1. Create command file
    let cmd_file = parent_dir.join(format!("src/commands/{command_snake}.rs"));
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

    // 2. Update commands/mod.rs
    let mod_path = parent_dir.join("src/commands/mod.rs");
    let mod_contents = deps.read_to_string(&mod_path)?;
    let new_mod = create_command::insert_command_mod(&mod_contents, &command_snake);
    deps.write(&mod_path, &new_mod)?;

    // 3. Update tyt_{parent_snake}.rs
    let enum_path = parent_dir.join(format!("src/tyt_{parent_snake}.rs"));
    let enum_contents = deps.read_to_string(&enum_path)?;
    let new_enum = create_command::insert_enum_variant(&enum_contents, name, command)?;
    deps.write(&enum_path, &new_enum)?;

    deps.write_stdout(
        format!("Added `{name}` command (`{command}`) to tyt-{parent}.\n").as_bytes(),
    )?;

    Ok(())
}
