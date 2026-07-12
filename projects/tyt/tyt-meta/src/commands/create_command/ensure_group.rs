use crate::{Dependencies, Error, Result, commands::create_command};
use std::path::{Path, PathBuf};

/// Ensures a parent command group exists as a subdirectory of `parent_dir` and
/// returns its subcommand-enum file (where child commands are wired) and its
/// directory (the parent of the next level down).
///
/// A group `g` under `parent_dir` lives at `parent_dir/<g>/` with its own
/// `mod.rs`, a `{group}.rs` `Parser` struct, and a `{group}_command.rs`
/// `Subcommand` enum. `<g>` is the bare segment; the files keep the
/// ancestor-prefixed snake name so leaves sharing a CLI name under different
/// groups stay distinct once flattened.
///
/// - If the group already exists (its enum file is present), it is returned
///   untouched.
/// - If nothing occupies the name, the group is scaffolded: its directory,
///   struct, enum, and `mod.rs` are written, the directory is registered in
///   `parent_mod`, and the struct is wired as a variant into `parent_enum`.
/// - If a leaf command or other module already occupies the name, this errors
///   rather than overwrite it; converting a leaf into a group is a manual step.
///
/// `ancestors` is the chain of group segments above this one, used to build the
/// prefixed type and file names.
pub fn ensure_group(
    deps: &impl Dependencies,
    parent_dir: &Path,
    parent_mod: &Path,
    parent_enum: &Path,
    ancestors: &[String],
    segment: &str,
) -> Result<(PathBuf, PathBuf)> {
    let segment_snake = create_command::kebab_to_snake_case(segment);
    let group_snake = ancestors
        .iter()
        .map(|ancestor| create_command::kebab_to_snake_case(ancestor))
        .chain([segment_snake.clone()])
        .collect::<Vec<_>>()
        .join("_");
    let group_pascal: String = ancestors
        .iter()
        .map(|ancestor| create_command::kebab_to_pascal_case(ancestor))
        .chain([create_command::kebab_to_pascal_case(segment)])
        .collect();

    let dir = parent_dir.join(&segment_snake);
    let struct_path = dir.join(format!("{group_snake}.rs"));
    let enum_path = dir.join(format!("{group_snake}_command.rs"));
    let mod_path = dir.join("mod.rs");

    // The group already exists: leave it and its wiring untouched.
    if enum_path.is_file() {
        return Ok((enum_path, dir));
    }

    // The directory is occupied by something that is not a command group (e.g. a
    // leaf promoted to a directory because it owns types). Do not overwrite it.
    if dir.exists() {
        return Err(Error::Meta(format!(
            "`{segment}` already exists at {} but is not a command group (no {group_snake}_command.rs). \
             Converting it into a group is a manual step.",
            dir.display()
        )));
    }

    // A flat leaf command already occupies the name. Converting a leaf into a
    // group means relocating it into a directory, which is a manual step.
    let flat_leaf = parent_dir.join(format!("{group_snake}.rs"));
    if flat_leaf.is_file() {
        return Err(Error::Meta(format!(
            "a leaf command already exists at {}. Converting a leaf into a group is a manual step: \
             move it into `{segment_snake}/` and add its {group_snake}_command.rs.",
            flat_leaf.display()
        )));
    }

    // Brand-new group: scaffold the directory, then register it upward.
    let description = format!("The `{segment}` command group.");
    deps.create_dir_all(&dir)?;
    deps.write(
        &struct_path,
        &create_command::group_struct_template(&group_pascal, segment, &description),
    )?;
    deps.write(
        &enum_path,
        &create_command::group_enum_template(&group_pascal, &description),
    )?;
    deps.write(&mod_path, &group_mod_source(&group_snake, &segment_snake))?;
    create_command::register_command_mod(deps, parent_mod, &segment_snake)?;
    create_command::wire_enum_variant(deps, parent_enum, &group_pascal, segment)?;

    Ok((enum_path, dir))
}

/// The `mod.rs` for a brand-new group directory: the struct and enum modules and
/// their re-exports. The struct module shares the directory's name only at the
/// first level, where `clippy::module_inception` needs allowing.
fn group_mod_source(group_snake: &str, segment_snake: &str) -> String {
    let inception = if group_snake == segment_snake {
        "#[allow(clippy::module_inception)]\n"
    } else {
        ""
    };
    format!(
        "{inception}mod {group_snake};\n\
         mod {group_snake}_command;\n\
         \n\
         pub use {group_snake}::*;\n\
         pub use {group_snake}_command::*;\n"
    )
}
