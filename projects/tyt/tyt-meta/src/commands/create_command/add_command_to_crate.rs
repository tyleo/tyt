use crate::{
    Dependencies, Error, Result,
    commands::{CreateCommand, create_command},
};

/// Adds a command to an existing crate, nesting it under zero or more parent
/// command groups.
///
/// `parents[0]` is the crate suffix (e.g. `vxl` for the `vxl` crate); any
/// remaining entries are parent command groups, created on demand so the new
/// command lands that many levels deep (e.g. `tyt vxl to vmax`). Each group is a
/// subdirectory of `commands/`, so the new leaf is written into the innermost
/// group's directory and registered in that directory's `mod.rs` and enum.
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

    // A grouped command's type and file are prefixed with its parent-group path
    // so leaves sharing a CLI name under different groups stay distinct once the
    // modules are flattened. The clap `#[command(name)]` keeps the bare CLI name.
    // Top-level commands (no groups) are left unprefixed.
    let group_pascal: String = groups
        .iter()
        .map(|group| create_command::kebab_to_pascal_case(group))
        .collect();
    let leaf_name = format!("{group_pascal}{name}");
    let leaf_snake = groups
        .iter()
        .map(|group| create_command::kebab_to_snake_case(group))
        .chain([command_snake])
        .collect::<Vec<_>>()
        .join("_");

    let root = deps.workspace_root()?;
    // Discover the crate on disk, inferring its group directory and prefix from
    // whatever `--dir` / `--prefix` were not given.
    let (dir, prefix) = create_command::find_parent_crate(
        deps,
        &root,
        crate_suffix,
        cmd.dir.as_deref(),
        cmd.prefix,
    )?;
    let naming = create_command::CrateNaming::new(prefix, crate_suffix);
    let crate_dir = root
        .join(create_command::PROJECTS_DIR)
        .join(&dir)
        .join(&naming.package);

    let commands_dir = crate_dir.join("src/commands");

    // Walk the parent groups from the crate root inward, ensuring each group
    // directory exists, and track the directory and enum the new command lands
    // in: the crate root enum and `commands/` itself when there are no groups,
    // otherwise the innermost group's directory and subcommand enum.
    let mut leaf_dir = commands_dir.clone();
    let mut enum_path = crate_dir.join(format!("src/{}.rs", naming.module));
    for (depth, group) in groups.iter().enumerate() {
        let parent_mod = leaf_dir.join("mod.rs");
        let (group_enum, group_dir) = create_command::ensure_group(
            deps,
            &leaf_dir,
            &parent_mod,
            &enum_path,
            &groups[..depth],
            group,
        )?;
        enum_path = group_enum;
        leaf_dir = group_dir;
    }
    let mod_path = leaf_dir.join("mod.rs");

    // 1. Create the leaf command file in the innermost group directory.
    let cmd_file = leaf_dir.join(format!("{leaf_snake}.rs"));
    if cmd_file.exists() {
        return Err(Error::Meta(format!(
            "command file already exists: {}",
            cmd_file.display()
        )));
    }
    deps.write(
        &cmd_file,
        &create_command::command_file_template(&leaf_name, command, description),
    )?;

    // 2. Register the module in that directory's mod.rs and wire the variant
    //    into the target enum.
    create_command::register_command_mod(deps, &mod_path, &leaf_snake)?;
    create_command::wire_enum_variant(deps, &enum_path, &leaf_name, command)?;

    // Prefixed crates run as `tyt {suffix} ...`; standalone crates run via their
    // own binary, e.g. `{suffix} ...`.
    let path = parents.join(" ");
    let binary = if prefix { "tyt " } else { "" };
    deps.write_stdout(
        format!("Added `{leaf_name}` command. Run it with `{binary}{path} {command}`.\n")
            .as_bytes(),
    )?;

    Ok(())
}
