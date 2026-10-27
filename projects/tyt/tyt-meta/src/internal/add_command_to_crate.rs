use crate::{
    CrateNaming, Dependencies, Error, PROJECTS_DIR, Result, command_file_template,
    commands::CreateCommand, group_enum_template, group_struct_template, kebab_to_pascal_case,
    kebab_to_snake_case, separate_members,
};
use std::{
    mem,
    path::{Path, PathBuf},
};

/// Adds a command to an existing crate, nesting it under zero or more parent
/// command groups.
///
/// `parents[0]` is the crate suffix (e.g. `vxl` for the `vxl` crate); any
/// remaining entries are parent command groups, created on demand so the new
/// command lands that many levels deep (e.g. `tyt vxl to vmax`). Each group is
/// a subdirectory of `commands/`, so the new leaf is written into the innermost
/// group's directory and registered in that directory's `mod.rs` and enum.
pub fn add_command_to_crate(
    cmd: &CreateCommand,
    deps: &impl Dependencies,
    parents: &[String],
) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let command_snake = kebab_to_snake_case(command);

    let (crate_suffix, groups) = parents
        .split_first()
        .ok_or_else(|| Error::Meta("at least one parent is required".to_string()))?;

    // A grouped command's type and file are prefixed with its parent-group path
    // so leaves sharing a CLI name under different groups stay distinct once
    // the modules are flattened. The clap `#[command(name)]` keeps the bare CLI
    // name. Top-level commands (no groups) are left unprefixed.
    let group_pascal: String = groups
        .iter()
        .map(|group| kebab_to_pascal_case(group))
        .collect();
    let leaf_name = format!("{group_pascal}{name}");
    let leaf_snake = groups
        .iter()
        .map(|group| kebab_to_snake_case(group))
        .chain([command_snake])
        .collect::<Vec<_>>()
        .join("_");

    let root = deps.workspace_root()?;
    // Discover the crate on disk, inferring its group directory and prefix from
    // whatever `--dir` / `--prefix` were not given.
    let (dir, prefix) =
        find_parent_crate(deps, &root, crate_suffix, cmd.dir.as_deref(), cmd.prefix)?;
    let naming = CrateNaming::new(prefix, crate_suffix);
    let crate_dir = root.join(PROJECTS_DIR).join(&dir).join(&naming.package);

    let commands_dir = crate_dir.join("src/commands");

    // Walk the parent groups from the crate root inward, ensuring each group
    // directory exists, and track the directory and enum the new command lands
    // in: the crate root enum and `commands/` itself when there are no groups,
    // otherwise the innermost group's directory and subcommand enum.
    let mut leaf_dir = commands_dir.clone();
    let mut enum_path = crate_dir.join(format!("src/{}.rs", naming.module));
    for (depth, group) in groups.iter().enumerate() {
        let parent_mod = leaf_dir.join("mod.rs");
        let (group_enum, group_dir) = ensure_group(
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
        &command_file_template(&leaf_name, command, description),
    )?;

    // 2. Register the module in that directory's mod.rs and wire the variant
    //    into the target enum.
    register_command_mod(deps, &mod_path, &leaf_snake)?;
    wire_enum_variant(deps, &enum_path, &leaf_name, command)?;

    // Prefixed crates run as `tyt {suffix} ...`; standalone crates run via
    // their own binary, e.g. `{suffix} ...`.
    let path = parents.join(" ");
    let binary = if prefix { "tyt " } else { "" };
    deps.write_stdout(
        format!("Added `{leaf_name}` command. Run it with `{binary}{path} {command}`.\n")
            .as_bytes(),
    )?;

    Ok(())
}

/// Locates an existing command crate for `suffix`, returning its `projects/`
/// group directory and whether its name carries the `tyt-` prefix.
///
/// `dir` and `prefix` narrow the search when given; whatever is left out is
/// inferred by looking on disk for a scaffolded crate, recognized by its root
/// command-enum file `src/{module}.rs`. That file also pins down the prefix,
/// so a command crate like `tyt-vmax` is never confused with a same-named
/// library crate such as the `vmax` codec. The search covers each group and
/// `projects/` itself, which it returns as an empty group, so a crate like
/// `projects/vxl` is found too. Errors when nothing matches or when more than
/// one crate does.
fn find_parent_crate(
    deps: &impl Dependencies,
    root: &Path,
    suffix: &str,
    dir: Option<&str>,
    prefix: Option<bool>,
) -> Result<(String, bool)> {
    let projects = root.join(PROJECTS_DIR);

    let dirs: Vec<String> = match dir {
        Some(dir) => vec![dir.to_string()],

        None => {
            let mut groups: Vec<String> = deps
                .read_dir(&projects)?
                .into_iter()
                .filter(|path| path.is_dir())
                .filter_map(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(String::from)
                })
                .collect();
            groups.sort();
            groups.insert(0, String::new());
            groups
        }
    };
    let prefixes: Vec<bool> = match prefix {
        Some(prefix) => vec![prefix],
        None => vec![true, false],
    };

    let mut matches: Vec<(String, bool)> = Vec::new();
    for group in &dirs {
        for &prefixed in &prefixes {
            let naming = CrateNaming::new(prefixed, suffix);
            let enum_file = projects
                .join(group)
                .join(&naming.package)
                .join(format!("src/{}.rs", naming.module));
            if enum_file.is_file() {
                matches.push((group.clone(), prefixed));
            }
        }
    }

    match matches.as_slice() {
        [] => Err(Error::Meta(format!(
            "parent crate `{suffix}` not found under {}",
            projects.display()
        ))),

        [only] => Ok(only.clone()),

        many => {
            let locations: Vec<String> = many
                .iter()
                .map(|(group, prefixed)| {
                    Path::new(PROJECTS_DIR)
                        .join(group)
                        .join(CrateNaming::new(*prefixed, suffix).package)
                        .display()
                        .to_string()
                })
                .collect();
            Err(Error::Meta(format!(
                "parent crate `{suffix}` is ambiguous; matched {}. Specify --dir and/or --prefix.",
                locations.join(", ")
            )))
        }
    }
}

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
fn ensure_group(
    deps: &impl Dependencies,
    parent_dir: &Path,
    parent_mod: &Path,
    parent_enum: &Path,
    ancestors: &[String],
    segment: &str,
) -> Result<(PathBuf, PathBuf)> {
    let segment_snake = kebab_to_snake_case(segment);
    let group_snake = ancestors
        .iter()
        .map(|ancestor| kebab_to_snake_case(ancestor))
        .chain([segment_snake.clone()])
        .collect::<Vec<_>>()
        .join("_");
    let group_pascal: String = ancestors
        .iter()
        .map(|ancestor| kebab_to_pascal_case(ancestor))
        .chain([kebab_to_pascal_case(segment)])
        .collect();

    let dir = parent_dir.join(&segment_snake);
    let struct_path = dir.join(format!("{group_snake}.rs"));
    let enum_path = dir.join(format!("{group_snake}_command.rs"));
    let mod_path = dir.join("mod.rs");

    // The group already exists: leave it and its wiring untouched.
    if enum_path.is_file() {
        return Ok((enum_path, dir));
    }

    // Something other than a command group occupies the directory (e.g. a
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
    let about = format!("Groups the `{segment}` commands.");
    let description = format!("The `{segment}` command group.");
    deps.create_dir_all(&dir)?;
    deps.write(
        &struct_path,
        &group_struct_template(&group_pascal, segment, &about),
    )?;
    deps.write(
        &enum_path,
        &group_enum_template(&group_pascal, &description),
    )?;
    deps.write(&mod_path, &group_mod_source(&group_snake, &segment_snake))?;
    register_command_mod(deps, parent_mod, &segment_snake)?;
    wire_enum_variant(deps, parent_enum, &group_pascal, segment)?;

    Ok((enum_path, dir))
}

/// The `mod.rs` for a brand-new group directory: the struct and enum modules
/// and their re-exports. The struct module shares the directory's name only at
/// the first level, where `clippy::module_inception` needs allowing.
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

/// Reads a `commands/mod.rs`, inserts a `mod`/`pub use` entry for `module`, and
/// writes it back.
fn register_command_mod(deps: &impl Dependencies, mod_path: &Path, module: &str) -> Result<()> {
    let contents = deps.read_to_string(mod_path)?;
    let updated = insert_command_mod(&contents, module);
    deps.write(mod_path, &updated)
}

/// Inserts a `mod {module};` and a `pub use {module}::*;` line into a
/// `commands/mod.rs`, each at its sorted position, and returns the new source.
///
/// Purely additive: every existing line is preserved verbatim, including doc
/// comments, blank lines, `#[allow(...)]` attributes, and `pub(crate) use`
/// re-exports. If `module` is already declared, the source is returned
/// unchanged.
fn insert_command_mod(contents: &str, module: &str) -> String {
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
    let trailing_newline = contents.is_empty() || contents.ends_with('\n');
    let has_pub_use = lines.iter().any(|line| pub_use_name(line).is_some());

    insert_sorted_line(&mut lines, &format!("mod {module};"), module, mod_name);
    // A freshly scaffolded commands/mod.rs starts empty; give it the
    // two-section blank separator before its first re-export.
    if !has_pub_use && lines.last().is_some_and(|line| !line.trim().is_empty()) {
        lines.push(String::new());
    }
    insert_sorted_line(
        &mut lines,
        &format!("pub use {module}::*;"),
        module,
        pub_use_name,
    );

    let mut result = lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    result
}

/// The module name in a `mod {name};` line, ignoring leading whitespace.
fn mod_name(line: &str) -> Option<&str> {
    line.trim().strip_prefix("mod ")?.strip_suffix(';')
}

/// The module name in a `pub use {name}::*;` line, ignoring leading whitespace.
fn pub_use_name(line: &str) -> Option<&str> {
    line.trim().strip_prefix("pub use ")?.strip_suffix("::*;")
}

/// Inserts `new_line` among the lines matching `key`, keeping them sorted by
/// the key. Skips insertion when `name` is already present. An insertion before
/// an attributed line lands above the attribute so it stays with its module.
fn insert_sorted_line(
    lines: &mut Vec<String>,
    new_line: &str,
    name: &str,
    key: impl Fn(&str) -> Option<&str>,
) {
    let matches: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| key(line).map(|found| (index, found)))
        .collect();

    if matches.iter().any(|(_, found)| *found == name) {
        return;
    }

    let insert_at = match matches.iter().find(|(_, found)| *found > name) {
        // Before the first later peer, above any attributes bound to it.
        Some(&(index, _)) => {
            let mut at = index;
            while at > 0 && lines[at - 1].trim_start().starts_with("#[") {
                at -= 1;
            }
            at
        }

        // After the last peer, or at the end when there are none.
        None => matches
            .last()
            .map(|&(index, _)| index + 1)
            .unwrap_or(lines.len()),
    };

    lines.insert(insert_at, new_line.to_string());
}

/// Reads a subcommand enum source file, inserts a variant + match arm for
/// `name`/`command`, and writes it back.
fn wire_enum_variant(
    deps: &impl Dependencies,
    enum_path: &Path,
    name: &str,
    command: &str,
) -> Result<()> {
    let contents = deps.read_to_string(enum_path)?;
    let updated = insert_enum_variant(&contents, name, command)?;
    deps.write(enum_path, &updated)
}

/// Wires a child command into a subcommand-enum source file: imports its type,
/// adds its `#[command(name = "...")]` variant, and adds its `match self` arm,
/// each at its sorted position.
///
/// Purely additive: existing variants, arms, imports, comments, and formatting
/// are preserved. A child already wired is left unchanged.
fn insert_enum_variant(contents: &str, name: &str, command: &str) -> Result<String> {
    let contents = insert_commands_use(contents, name);

    let enum_name = contents
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pub enum ")
                .and_then(|rest| rest.split([' ', '{']).next())
        })
        .ok_or_else(|| Error::Meta("no `pub enum` found in the subcommand enum file".into()))?
        .to_string();

    let deps_param = contents
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pub fn execute(self,")
                .and_then(|rest| rest.split(':').next())
                .map(str::trim)
        })
        .unwrap_or("dependencies")
        .to_string();

    let binding = command.replace('-', "_");
    let trailing_newline = contents.is_empty() || contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();

    let variant = [
        format!("    #[command(name = \"{command}\")]"),
        format!("    {name}({name}),"),
    ];
    insert_into_braces(
        &mut lines,
        |line| line.starts_with("pub enum "),
        variant_key,
        name,
        &variant,
    )
    .ok_or_else(|| Error::Meta("could not place the enum variant".into()))?;
    separate_members(&mut lines, |line| line.starts_with("pub enum "));

    let arm = [format!(
        "            {enum_name}::{name}({binding}) => {binding}.execute({deps_param}),"
    )];
    insert_into_braces(
        &mut lines,
        |line| line.starts_with("match self"),
        arm_key,
        name,
        &arm,
    )
    .ok_or_else(|| Error::Meta("could not place the match arm".into()))?;

    let mut result = lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    Ok(result)
}

/// The variant name in an enum-body line such as `Foo(Foo),`, or `None` for an
/// attribute, blank, or brace line.
fn variant_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("#[") || trimmed == "}" {
        return None;
    }
    Some(
        trimmed
            .split(['(', ',', ' ', '{'])
            .next()
            .unwrap_or(trimmed),
    )
}

/// The variant name in a match arm such as `Enum::Foo(foo) => ...`, or `None`
/// for an attribute, blank, or brace line.
fn arm_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("#[") || trimmed == "}" {
        return None;
    }
    let after = trimmed.split("::").nth(1)?;
    Some(after.split('(').next().unwrap_or(after))
}

/// Splices `name` into the file's `use crate::...` import at its sorted place
/// among the `commands::` names, whether the import is `use crate::commands::`
/// or a `use crate::{...}` tree holding a `commands` entry. Adds the import
/// when absent and leaves it unchanged when `name` is already present.
fn insert_commands_use(contents: &str, name: &str) -> String {
    let trailing_newline = contents.is_empty() || contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();

    let render = |lines: Vec<String>| {
        let mut result = lines.join("\n");
        if trailing_newline {
            result.push('\n');
        }
        result
    };

    // A `use crate::commands::` import wins over an earlier `use crate::` one.
    let import_at = |prefix: &str| {
        lines
            .iter()
            .position(|line| line.trim_start().starts_with(prefix))
    };
    let Some(start) = import_at("use crate::commands").or_else(|| import_at("use crate::")) else {
        let at = lines
            .iter()
            .position(|line| line.trim_start().starts_with("use "))
            .unwrap_or(0);
        lines.insert(at, format!("use crate::commands::{{{name}}};"));
        return render(lines);
    };

    // The statement runs from `start` to the first line closing it with `;`.
    let end = (start..lines.len())
        .find(|&index| lines[index].contains(';'))
        .unwrap_or(start);

    // Collect the crate's entries across however many lines the tree spans.
    let statement = lines[start..=end].join(" ");
    let tree = statement
        .trim()
        .trim_start_matches("use crate::")
        .trim_end_matches(';')
        .trim();
    let mut entries = match tree.strip_prefix('{') {
        Some(inner) => split_use_entries(inner.strip_suffix('}').unwrap_or(inner)),
        None => vec![tree.to_string()],
    };

    // A bare `commands` entry imports the module itself, kept as `self`.
    let slot = entries
        .iter()
        .position(|entry| entry == "commands" || entry.starts_with("commands::"));
    let mut names = match slot.map(|index| entries[index].as_str()) {
        Some("commands") => vec!["self".to_string()],

        Some(entry) => {
            let rest = entry.trim_start_matches("commands::");
            match rest.strip_prefix('{') {
                Some(inner) => split_use_entries(inner.strip_suffix('}').unwrap_or(inner)),
                None => vec![rest.to_string()],
            }
        }

        None => Vec::new(),
    };

    if names.iter().any(|existing| existing == name) {
        return render(lines);
    }

    names.push(name.to_string());
    names.sort_by(|a, b| (a != "self", a).cmp(&(b != "self", b)));
    let commands = format!("commands::{{{}}}", names.join(", "));
    match slot {
        Some(index) => entries[index] = commands,
        None => entries.push(commands),
    }

    let statement = match entries.as_slice() {
        [only] => format!("use crate::{only};"),
        _ => format!("use crate::{{{}}};", entries.join(", ")),
    };
    lines.splice(start..=end, [statement]);
    render(lines)
}

/// Splits a use tree's brace contents into its top-level entries, trimmed and
/// with whitespace runs collapsed, skipping the empty entry a trailing comma
/// leaves.
fn split_use_entries(inner: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for character in inner.chars() {
        match character {
            '{' => depth += 1,

            '}' => depth -= 1,

            ',' if depth == 0 => {
                entries.push(mem::take(&mut current));
                continue;
            }

            _ => {}
        }
        current.push(character);
    }
    entries.push(current);
    entries
        .iter()
        .map(|entry| entry.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|entry| !entry.is_empty())
        .collect()
}

/// Inserts `entry` (its lines already indented) into the brace-delimited body
/// whose opening line matches `header`, keeping entries sorted by `key`.
/// Handles an empty `{}` body by expanding it. Returns `None` when no header
/// line is found, `Some(())` otherwise (including a no-op when `new_key` is
/// present).
fn insert_into_braces(
    lines: &mut Vec<String>,
    header: impl Fn(&str) -> bool,
    key: impl Fn(&str) -> Option<&str>,
    new_key: &str,
    entry: &[String],
) -> Option<()> {
    let open = lines.iter().position(|line| header(line.trim()))?;
    let indent: String = lines[open]
        .chars()
        .take_while(|character| character.is_whitespace())
        .collect();

    // An empty `{}` body: expand it around the new entry.
    if let Some(brace) = lines[open].find("{}") {
        let head = lines[open][..brace].to_string();
        let mut replacement = vec![format!("{head}{{")];
        replacement.extend(entry.iter().cloned());
        replacement.push(format!("{indent}}}"));
        lines.splice(open..=open, replacement);
        return Some(());
    }

    // Find the matching close brace by tracking depth from the opening line.
    let mut depth = 0i32;
    let mut close = open;
    'outer: for (index, line) in lines.iter().enumerate().skip(open) {
        for character in line.chars() {
            match character {
                '{' => depth += 1,

                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = index;
                        break 'outer;
                    }
                }

                _ => {}
            }
        }
    }

    // Already wired: leave it be.
    if lines[open + 1..close]
        .iter()
        .filter_map(|line| key(line))
        .any(|existing| existing == new_key)
    {
        return Some(());
    }

    // Insert before the first later entry (above its attributes), else before
    // the closing brace.
    let mut insert_at = close;
    for index in open + 1..close {
        if let Some(found) = key(&lines[index])
            && found > new_key
        {
            let mut at = index;
            while at > open + 1 && lines[at - 1].trim_start().starts_with("#[") {
                at -= 1;
            }
            insert_at = at;
            break;
        }
    }

    for (offset, line) in entry.iter().enumerate() {
        lines.insert(insert_at + offset, line.clone());
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use crate::{
        group_enum_template,
        internal::add_command_to_crate::{
            insert_command_mod, insert_commands_use, insert_enum_variant,
        },
    };

    #[test]
    fn inserts_a_mod_and_pub_use_in_sorted_position() {
        let source = "mod alpha;\nmod gamma;\n\npub use alpha::*;\npub use gamma::*;\n";
        let expected = "mod alpha;\nmod beta;\nmod gamma;\n\npub use alpha::*;\npub use beta::*;\npub use gamma::*;\n";
        assert_eq!(insert_command_mod(source, "beta"), expected);
    }

    #[test]
    fn appends_when_alphabetically_last() {
        let source = "mod alpha;\n\npub use alpha::*;\n";
        let expected = "mod alpha;\nmod zed;\n\npub use alpha::*;\npub use zed::*;\n";
        assert_eq!(insert_command_mod(source, "zed"), expected);
    }

    #[test]
    fn preserves_module_doc_attributes_and_pub_crate_use() {
        // A nested group mod.rs: an inner doc line, a module_inception
        // attribute, and a pub(crate) re-export must all survive untouched.
        let source = "//! Module docs.\n\n#[allow(clippy::module_inception)]\nmod to;\nmod to_goxl;\n\npub use to::*;\npub(crate) use to_goxl::*;\n";
        let expected = "//! Module docs.\n\n#[allow(clippy::module_inception)]\nmod to;\nmod to_command;\nmod to_goxl;\n\npub use to::*;\npub use to_command::*;\npub(crate) use to_goxl::*;\n";
        assert_eq!(insert_command_mod(source, "to_command"), expected);
    }

    #[test]
    fn inserting_before_an_attributed_module_lands_above_the_attribute() {
        // `mod bravo` sorts before the attributed `mod charlie`; the new line
        // must not split the attribute from its module.
        let source =
            "mod alpha;\n#[cfg(test)]\nmod charlie;\n\npub use alpha::*;\npub use charlie::*;\n";
        let expected = "mod alpha;\nmod bravo;\n#[cfg(test)]\nmod charlie;\n\npub use alpha::*;\npub use bravo::*;\npub use charlie::*;\n";
        assert_eq!(insert_command_mod(source, "bravo"), expected);
    }

    #[test]
    fn a_duplicate_module_is_a_no_op() {
        let source = "mod alpha;\nmod beta;\n\npub use alpha::*;\npub use beta::*;\n";
        assert_eq!(insert_command_mod(source, "beta"), source);
    }

    #[test]
    fn the_first_entry_in_an_empty_file_gets_a_section_separator() {
        // A freshly scaffolded commands/mod.rs is empty; the first command must
        // still produce the two-section mod / pub-use shape.
        assert_eq!(
            insert_command_mod("", "run"),
            "mod run;\n\npub use run::*;\n"
        );

        // A second command slots into both sections, keeping the blank.
        let after_run = insert_command_mod("", "run");
        assert_eq!(
            insert_command_mod(&after_run, "widget"),
            "mod run;\nmod widget;\n\npub use run::*;\npub use widget::*;\n"
        );
    }

    /// An empty subcommand enum as `group_enum_template` renders it.
    fn empty_enum() -> String {
        group_enum_template("To", "The `to` command group.")
    }

    #[test]
    fn wires_the_first_child_into_an_empty_enum() {
        let wired = insert_enum_variant(&empty_enum(), "ToGoxl", "goxl").unwrap();
        assert!(wired.contains("use crate::{Dependencies, Result, commands::{ToGoxl}};"));
        assert!(wired.contains("    #[command(name = \"goxl\")]\n    ToGoxl(ToGoxl),\n"));
        assert!(
            wired.contains("            ToCommand::ToGoxl(goxl) => goxl.execute(_dependencies),\n")
        );
        // The empty braces are gone.
        assert!(!wired.contains("pub enum ToCommand {}"));
        assert!(!wired.contains("match self {}"));
    }

    #[test]
    fn inserts_a_second_child_in_sorted_position() {
        let once = insert_enum_variant(&empty_enum(), "ToVmax", "vmax").unwrap();
        let twice = insert_enum_variant(&once, "ToGoxl", "goxl").unwrap();
        // Import stays a single sorted list.
        assert!(twice.contains("use crate::{Dependencies, Result, commands::{ToGoxl, ToVmax}};"));
        // Goxl sorts before Vmax in both the enum and the match.
        let goxl = twice.find("ToGoxl(ToGoxl)").unwrap();
        let vmax = twice.find("ToVmax(ToVmax)").unwrap();
        assert!(goxl < vmax);
        let goxl_arm = twice.find("ToCommand::ToGoxl").unwrap();
        let vmax_arm = twice.find("ToCommand::ToVmax").unwrap();
        assert!(goxl_arm < vmax_arm);
    }

    #[test]
    fn splices_into_a_multiline_import_by_comma_not_line() {
        // rustfmt may reflow the import across lines between runs; the splice
        // must still find the comma-separated names.
        let source = "use crate::commands::{\n    ToGoxl, ToVmax, ToVoxj,\n};\nuse clap::Subcommand;\n\n#[derive(Clone, Debug, Subcommand)]\npub enum ToCommand {\n    #[command(name = \"goxl\")]\n    ToGoxl(ToGoxl),\n    #[command(name = \"vmax\")]\n    ToVmax(ToVmax),\n    #[command(name = \"voxj\")]\n    ToVoxj(ToVoxj),\n}\n\nimpl ToCommand {\n    pub fn execute(self, dependencies: impl crate::Dependencies) -> crate::Result<()> {\n        match self {\n            ToCommand::ToGoxl(goxl) => goxl.execute(dependencies),\n            ToCommand::ToVmax(vmax) => vmax.execute(dependencies),\n            ToCommand::ToVoxj(voxj) => voxj.execute(dependencies),\n        }\n    }\n}\n".to_string();
        let wired = insert_enum_variant(&source, "ToMvox", "mvox").unwrap();
        assert!(wired.contains("use crate::commands::{ToGoxl, ToMvox, ToVmax, ToVoxj};"));
        assert!(wired.contains("    #[command(name = \"mvox\")]\n    ToMvox(ToMvox),\n"));
        assert!(wired.contains("ToCommand::ToMvox(mvox) => mvox.execute(dependencies),"));
    }

    #[test]
    fn splices_into_the_commands_entry_of_a_multiline_crate_tree() {
        let source = "use crate::{\n    Dependencies, Result,\n    commands::{ToGoxl, ToVoxj},\n};\nuse clap::Subcommand;\n";
        assert_eq!(
            insert_commands_use(source, "ToMvox"),
            "use crate::{Dependencies, Result, commands::{ToGoxl, ToMvox, ToVoxj}};\nuse clap::Subcommand;\n"
        );
    }

    #[test]
    fn a_bare_commands_module_import_keeps_self() {
        let source = "use crate::{Dependencies, Result, commands};\nuse clap::Subcommand;\n";
        assert_eq!(
            insert_commands_use(source, "Pixelate"),
            "use crate::{Dependencies, Result, commands::{self, Pixelate}};\nuse clap::Subcommand;\n"
        );
    }

    #[test]
    fn a_commands_import_wins_over_an_earlier_crate_import() {
        let source = "use crate::Dependencies;\nuse crate::commands::Img;\n";
        assert_eq!(
            insert_commands_use(source, "Edit"),
            "use crate::Dependencies;\nuse crate::commands::{Edit, Img};\n"
        );
    }

    #[test]
    fn re_wiring_an_existing_child_is_a_no_op() {
        let once = insert_enum_variant(&empty_enum(), "ToGoxl", "goxl").unwrap();
        let twice = insert_enum_variant(&once, "ToGoxl", "goxl").unwrap();
        assert_eq!(once, twice);
    }
}
