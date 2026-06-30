use crate::{Dependencies, Error, Result, commands::create_command};
use std::path::{Path, PathBuf};

/// Ensures a parent command group exists in a crate's `commands/` directory and
/// returns the path to its subcommand enum file (where child commands are wired).
///
/// - If nothing occupies the name, scaffolds the group's `Parser` struct +
///   `Subcommand` enum and wires the struct into `parent_enum_path` as a variant.
/// - If a pristine leaf-command stub already occupies the name, converts it into a
///   group in place (its variant in `parent_enum_path` is reused unchanged).
/// - If the group already exists, leaves it untouched.
///
/// `ancestors` is the chain of parent groups above this one; the group's type and
/// file names are prefixed with it so nested groups stay distinct in the flat
/// `commands/` namespace. The clap `#[command(name)]` keeps the bare CLI name.
pub fn ensure_group(
    deps: &impl Dependencies,
    commands_dir: &Path,
    mod_path: &Path,
    parent_enum_path: &Path,
    ancestors: &[String],
    group: &str,
) -> Result<PathBuf> {
    let group_snake = ancestors
        .iter()
        .map(|ancestor| create_command::kebab_to_snake_case(ancestor))
        .chain([create_command::kebab_to_snake_case(group)])
        .collect::<Vec<_>>()
        .join("_");
    let group_pascal: String = ancestors
        .iter()
        .map(|ancestor| create_command::kebab_to_pascal_case(ancestor))
        .chain([create_command::kebab_to_pascal_case(group)])
        .collect();
    let struct_path = commands_dir.join(format!("{group_snake}.rs"));
    let enum_path = commands_dir.join(format!("{group_snake}_command.rs"));
    let enum_mod = format!("{group_snake}_command");
    let description = format!("The `{group}` command group.");

    if !struct_path.exists() {
        // Brand-new group: scaffold the struct + enum and wire the struct into its parent.
        deps.write(
            &struct_path,
            &create_command::group_struct_template(&group_pascal, group, &description),
        )?;
        deps.write(
            &enum_path,
            &create_command::group_enum_template(&group_pascal, &description),
        )?;
        create_command::register_command_mod(deps, mod_path, &group_snake)?;
        create_command::register_command_mod(deps, mod_path, &enum_mod)?;
        create_command::wire_enum_variant(deps, parent_enum_path, &group_pascal, group)?;
    } else if !enum_path.exists() {
        // A leaf command already occupies this name — convert it into a group. Its
        // variant in the parent enum is already present and stays unchanged.
        convert_leaf_to_group(deps, &struct_path, &group_pascal, group)?;
        deps.write(
            &enum_path,
            &create_command::group_enum_template(&group_pascal, &description),
        )?;
        create_command::register_command_mod(deps, mod_path, &enum_mod)?;
    }

    Ok(enum_path)
}

/// Rewrites a pristine leaf-command stub as a group struct, preserving its
/// description. Errors if the command has been customized.
fn convert_leaf_to_group(
    deps: &impl Dependencies,
    struct_path: &Path,
    name: &str,
    command: &str,
) -> Result<()> {
    let contents = deps.read_to_string(struct_path)?;
    let description = stub_description(&contents, name, command).ok_or_else(|| {
        Error::Meta(format!(
            "command `{command}` already exists with custom code and cannot be \
             converted into a parent group automatically: {}",
            struct_path.display()
        ))
    })?;
    deps.write(
        struct_path,
        &create_command::group_struct_template(name, command, &description),
    )
}

/// Returns the description if `contents` is an unmodified leaf-command stub (safe
/// to regenerate as a group), or `None` if it has been customized.
fn stub_description(contents: &str, name: &str, command: &str) -> Option<String> {
    let description = contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("/// "))?
        .to_string();
    let expected = create_command::command_file_template(name, command, &description);
    (contents.trim_end() == expected.trim_end()).then_some(description)
}
