use crate::{
    CrateNaming, DEPENDENCIES_IMPL_RS_TEMPLATE, DEPENDENCIES_RS_TEMPLATE, Dependencies,
    ERROR_RS_TEMPLATE, Error, LICENSE_TEMPLATE, PROJECTS_DIR, RESULT_RS_TEMPLATE, Result,
    add_command_to_crate, cargo_toml_template, kebab_to_snake_case, lib_rs_template,
    main_rs_template, readme_template, separate_members, tyt_enum_template_empty,
};
use clap::{ArgAction, Parser};
use std::path::Path;

/// Scaffolds a new sub-crate or adds a command to an existing one.
///
/// Without `--parent`, creates a brand-new sub-crate with all boilerplate and
/// adds it to the workspace. By default the crate is named `tyt-<command>`,
/// placed at `projects/tyt/tyt-<command>`, and wired into the top-level `tyt`
/// binary. Pass `--dir` to place it under a different `projects/` group and
/// `--prefix false` to drop the `tyt-` name prefix, which produces a standalone
/// crate that is added to the workspace but not the `tyt` binary.
///
/// With `--parent`, adds a command to an existing sub-crate. The first
/// `--parent` is the crate suffix; repeat it to nest the command under parent
/// command groups, which are created on demand (e.g.
/// `--parent voxj --parent from` adds a command at `tyt voxj from <command>`).
/// Here `--dir` and `--prefix` are inferred from the existing crate when
/// omitted, and only needed to disambiguate same-named crates.
///
/// Each parent group is prepended to the new command's type and file name, so
/// `show` under `palette` becomes `PaletteShow` in `palette_show.rs` while its
/// CLI name stays `show`. This keeps same-named leaves under different groups
/// distinct.
#[derive(Clone, Debug, Parser)]
#[command(name = "create-command")]
pub struct CreateCommand {
    /// PascalCase type name (e.g. `FooBar`). Parent groups are prepended, so
    /// `Show` under `palette` lands as `PaletteShow`.
    #[arg(value_name = "name")]
    pub name: String,

    /// kebab-case CLI name (e.g. `foo-bar`).
    #[arg(value_name = "command")]
    pub command: String,

    /// Description for doc comments, `Cargo.toml`, and `README.md`.
    #[arg(value_name = "description")]
    pub description: String,

    /// Parent command path, from crate suffix inward (e.g. `fbx` for
    /// `tyt-fbx`). Repeatable; each repeat nests one group deeper (e.g.
    /// `-p voxj -p from`).
    #[arg(value_name = "parent", short, long)]
    pub parent: Vec<String>,

    /// Group directory under `projects/` to place the crate in (e.g.
    /// `vxl`). Defaults to `tyt` when creating; inferred from disk when
    /// adding with `--parent`.
    #[arg(value_name = "dir", long)]
    pub dir: Option<String>,

    /// Applies the `tyt-` crate and `Tyt` enum name prefix. `--prefix false`
    /// makes a standalone crate not wired into the `tyt` binary. Defaults to
    /// `true` when creating; inferred from disk when adding with `--parent`.
    #[arg(value_name = "prefix", long, action = ArgAction::Set)]
    pub prefix: Option<bool>,
}

impl CreateCommand {
    /// Creates the crate, or adds the command to its `--parent` crate.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self.parent.as_slice() {
            [] => create_crate(&self, &dependencies),
            parents => add_command_to_crate(&self, &dependencies, parents),
        }
    }
}

/// Workspace-relative group directory of the `tyt` binary crate, which lives at
/// `{TYT_PROJECT_DIR}/tyt`. Prefixed sub-crates are wired into that binary.
const TYT_PROJECT_DIR: &str = "projects/tyt";

/// Scaffolds a new sub-crate and wires it into the workspace, and into the
/// `tyt` binary when the crate carries the `tyt-` prefix.
fn create_crate(cmd: &CreateCommand, deps: &impl Dependencies) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let dir = cmd.dir.as_deref().unwrap_or("tyt");
    let prefix = cmd.prefix.unwrap_or(true);
    let naming = CrateNaming::new(prefix, command);
    let root_enum = naming.root_enum(name);
    let root = deps.workspace_root()?;
    let crate_dir = root.join(PROJECTS_DIR).join(dir).join(&naming.package);

    if crate_dir.exists() {
        return Err(Error::Meta(format!(
            "crate directory already exists: {}",
            crate_dir.display()
        )));
    }

    let src = crate_dir.join("src");
    let commands_dir = src.join("commands");
    deps.create_dir_all(&commands_dir)?;

    // 1. Cargo.toml
    deps.write(
        crate_dir.join("Cargo.toml"),
        &cargo_toml_template(&naming.package, command, description),
    )?;

    // 2. LICENSE
    deps.write(crate_dir.join("LICENSE"), LICENSE_TEMPLATE)?;

    // 3. README.md
    deps.write(
        crate_dir.join("README.md"),
        &readme_template(&naming.package, name, description),
    )?;

    // 4. src/lib.rs
    deps.write(src.join("lib.rs"), &lib_rs_template(&naming.module))?;

    // 5. src/main.rs
    deps.write(
        src.join("main.rs"),
        &main_rs_template(&naming.module, &root_enum, command, description),
    )?;

    // 6. src/dependencies.rs
    deps.write(src.join("dependencies.rs"), DEPENDENCIES_RS_TEMPLATE)?;

    // 7. src/dependencies_impl.rs
    deps.write(
        src.join("dependencies_impl.rs"),
        DEPENDENCIES_IMPL_RS_TEMPLATE,
    )?;

    // 8. src/error.rs
    deps.write(src.join("error.rs"), ERROR_RS_TEMPLATE)?;

    // 9. src/result.rs
    deps.write(src.join("result.rs"), RESULT_RS_TEMPLATE)?;

    // 10. src/{module}.rs
    deps.write(
        src.join(format!("{}.rs", naming.module)),
        &tyt_enum_template_empty(&root_enum, description),
    )?;

    // 11. src/commands/mod.rs
    deps.write(commands_dir.join("mod.rs"), "")?;

    // Wire into existing files.

    // Workspace Cargo.toml
    wire_workspace_cargo_toml(deps, &root, dir, &naming.package)?;

    // Prefixed crates are tyt subcommands and are wired into the tyt binary so
    // `tyt {command}` runs them. Standalone (no-prefix) crates are left out.
    if prefix {
        // projects/tyt/tyt/Cargo.toml
        wire_tyt_cargo_toml(deps, &root, command)?;

        // projects/tyt/tyt/src/dependencies.rs
        wire_tyt_dependencies(deps, &root, command, name)?;

        // projects/tyt/tyt/src/dependencies_impl.rs
        wire_tyt_dependencies_impl(deps, &root, command, name)?;

        // projects/tyt/tyt/src/error.rs
        wire_tyt_error(deps, &root, command, name)?;

        // projects/tyt/tyt/src/tyt.rs
        wire_tyt_tyt_rs(deps, &root, command, name)?;
    }

    let location = format!("{}/{dir}/{}", PROJECTS_DIR, naming.package);
    let mut parent_flags = String::new();
    if dir != "tyt" {
        parent_flags.push_str(&format!(" --dir {dir}"));
    }
    if !prefix {
        parent_flags.push_str(" --prefix false");
    }
    let wiring = if prefix {
        format!("the workspace and the tyt binary as `tyt {command}`")
    } else {
        "the workspace".to_string()
    };
    deps.write_stdout(
        format!(
            "Created {package} crate at {location} and wired it into {wiring}.\n\
             Next: add commands with `tyt meta create-command <Name> <command> <desc> --parent {command}{parent_flags}`\n",
            package = naming.package
        )
        .as_bytes(),
    )?;

    Ok(())
}

/// Adds the crate to the workspace `members` list and `[patch.crates-io]`
/// table, each at its sorted position.
fn wire_workspace_cargo_toml(
    deps: &impl Dependencies,
    root: &Path,
    dir: &str,
    package: &str,
) -> Result<()> {
    let path = root.join("Cargo.toml");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let member_entry = format!("    \"{PROJECTS_DIR}/{dir}/{package}\",");
    let patch_entry = format!("{package} = {{ path = \"{PROJECTS_DIR}/{dir}/{package}\" }}");

    let mut member_inserted = false;
    let mut patch_inserted = false;
    let mut in_members = false;
    let mut in_patch = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed == "members = [" {
            in_members = true;
            result.push(line.to_string());
            continue;
        }

        if in_members && !member_inserted {
            if trimmed == "]" {
                result.push(member_entry.clone());
                member_inserted = true;
                in_members = false;
                result.push(line.to_string());
                continue;
            }
            // Check if we should insert before this line (sorted)
            if trimmed > member_entry.trim() && !member_inserted {
                result.push(member_entry.clone());
                member_inserted = true;
                in_members = false;
            }
            result.push(line.to_string());
            continue;
        }

        if trimmed == "[patch.crates-io]" {
            in_patch = true;
            result.push(line.to_string());
            continue;
        }

        if in_patch && !patch_inserted {
            if trimmed.is_empty() || trimmed.starts_with('[') {
                result.push(patch_entry.clone());
                patch_inserted = true;
                in_patch = false;
                result.push(line.to_string());
                continue;
            }
            if trimmed > patch_entry.trim() {
                result.push(patch_entry.clone());
                patch_inserted = true;
                in_patch = false;
            }
            result.push(line.to_string());
            continue;
        }

        result.push(line.to_string());
    }

    // If patch entry not yet inserted (end of file)
    if in_patch && !patch_inserted {
        result.push(patch_entry);
    }

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

/// Adds the crate to the `tyt` binary's dependencies and its `impl` feature.
fn wire_tyt_cargo_toml(deps: &impl Dependencies, root: &Path, command: &str) -> Result<()> {
    let path = root.join(TYT_PROJECT_DIR).join("tyt/Cargo.toml");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let dep_line = format!("tyt-{command} = {{ version = \"0.1.0\" }}");
    let feature_entry = format!("\"tyt-{command}/impl\"");

    let mut dep_inserted = false;
    let mut in_deps = false;
    let mut feature_handled = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed == "[dependencies]" {
            in_deps = true;
            result.push(line.to_string());
            continue;
        }

        if in_deps && !dep_inserted {
            if trimmed.is_empty() || trimmed.starts_with('[') {
                result.push(dep_line.clone());
                dep_inserted = true;
                in_deps = false;
                result.push(line.to_string());
                continue;
            }
            if trimmed > dep_line.as_str() {
                result.push(dep_line.clone());
                dep_inserted = true;
                in_deps = false;
            }
            result.push(line.to_string());
            continue;
        }

        // Handle impl feature line
        if !feature_handled && trimmed.starts_with("impl = [") {
            let start = line.find('[').unwrap() + 1;
            let end = line.rfind(']').unwrap();
            let existing = &line[start..end];
            let mut entries: Vec<&str> = existing
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            entries.push(&feature_entry);
            entries.sort();
            result.push(format!("impl = [{}]", entries.join(", ")));
            feature_handled = true;
            continue;
        }

        result.push(line.to_string());
    }

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

/// Adds the crate's associated type and accessor to the `tyt` binary's
/// `Dependencies` trait.
fn wire_tyt_dependencies(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root.join(TYT_PROJECT_DIR).join("tyt/src/dependencies.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = kebab_to_snake_case(command);
    let use_line = format!("use tyt_{snake}::Dependencies as Tyt{name}Dependencies;");
    let type_line = format!("    type Tyt{name}Dependencies: Tyt{name}Dependencies;");
    let method_line =
        format!("    fn tyt_{snake}_dependencies(&self) -> Self::Tyt{name}Dependencies;");

    let mut use_inserted = false;
    let mut type_inserted = false;
    let mut method_inserted = false;
    let mut in_trait = false;
    let mut past_types = false;

    for line in &lines {
        let trimmed = line.trim();

        // Insert use line in sorted position
        if !use_inserted && trimmed.starts_with("use ") {
            if trimmed > use_line.trim() {
                result.push(use_line.clone());
                use_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }
        if !use_inserted && !trimmed.starts_with("use ") && !result.is_empty() {
            // Past all use lines, so insert at the end of the use block
            result.push(use_line.clone());
            use_inserted = true;
        }

        if trimmed.starts_with("pub trait Dependencies") {
            in_trait = true;
            result.push(line.to_string());
            continue;
        }

        if in_trait {
            // Insert type in sorted position among types
            if !type_inserted && trimmed.starts_with("type ") {
                if trimmed > type_line.trim() {
                    result.push(type_line.clone());
                    type_inserted = true;
                }
                result.push(line.to_string());
                continue;
            }

            // Transition from types to methods
            if !type_inserted && trimmed.starts_with("fn ") {
                // The blank line separator is already in result; insert type
                // before it.
                if result.last().is_some_and(|l| l.trim().is_empty()) {
                    result.pop();
                }
                result.push(type_line.clone());
                result.push(String::new());
                type_inserted = true;
                past_types = true;
            }

            if type_inserted && !past_types && trimmed.is_empty() {
                past_types = true;
                result.push(line.to_string());
                continue;
            }

            // Insert method in sorted position among methods
            if past_types && !method_inserted && trimmed.starts_with("fn ") {
                if trimmed > method_line.trim() {
                    result.push(method_line.clone());
                    method_inserted = true;
                }
                result.push(line.to_string());
                continue;
            }

            if trimmed == "}" && !method_inserted {
                result.push(method_line.clone());
                method_inserted = true;
            }
        }

        result.push(line.to_string());
    }

    separate_members(&mut result, |line| {
        line.starts_with("pub trait Dependencies")
    });
    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

/// Adds the crate's associated type and accessor to the `tyt` binary's
/// `DependenciesImpl`.
fn wire_tyt_dependencies_impl(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root
        .join(TYT_PROJECT_DIR)
        .join("tyt/src/dependencies_impl.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = kebab_to_snake_case(command);
    let use_line = format!("use tyt_{snake}::DependenciesImpl as Tyt{name}DependenciesImpl;");
    let type_line = format!("    type Tyt{name}Dependencies = Tyt{name}DependenciesImpl;");
    let method_fn =
        format!("    fn tyt_{snake}_dependencies(&self) -> Self::Tyt{name}Dependencies {{");
    let method_body = format!("        Tyt{name}DependenciesImpl");
    let method_close = "    }".to_string();

    let mut use_inserted = false;
    let mut type_inserted = false;
    let mut method_inserted = false;
    let mut in_impl = false;
    let mut past_types = false;
    let mut impl_depth: i32 = 0;

    for line in &lines {
        let trimmed = line.trim();
        let brace_delta = trimmed.chars().filter(|&c| c == '{').count() as i32
            - trimmed.chars().filter(|&c| c == '}').count() as i32;

        // Insert use line in sorted position
        if !use_inserted && trimmed.starts_with("use tyt_") {
            if trimmed > use_line.trim() {
                result.push(use_line.clone());
                use_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }
        if !use_inserted
            && !trimmed.starts_with("use ")
            && result.iter().any(|l| l.starts_with("use "))
        {
            result.push(use_line.clone());
            use_inserted = true;
        }

        if trimmed.starts_with("impl Dependencies for") {
            in_impl = true;
            impl_depth = brace_delta;
            result.push(line.to_string());
            continue;
        }

        if in_impl {
            // Only inspect lines at impl top-level (depth 1)
            if impl_depth == 1 {
                if !type_inserted && trimmed.starts_with("type ") {
                    if trimmed > type_line.trim() {
                        result.push(type_line.clone());
                        type_inserted = true;
                    }
                    result.push(line.to_string());
                    impl_depth += brace_delta;
                    continue;
                }

                // The first method ends the types, so a type sorting after
                // them all goes just before it.
                if trimmed.starts_with("fn ") {
                    if !type_inserted {
                        if result.last().is_some_and(|l| l.trim().is_empty()) {
                            result.pop();
                        }
                        result.push(type_line.clone());
                        result.push(String::new());
                        type_inserted = true;
                    }
                    past_types = true;
                }

                if past_types && !method_inserted && trimmed.starts_with("fn ") {
                    let method_sort_key = format!("fn tyt_{snake}_dependencies");
                    if trimmed > method_sort_key.as_str() {
                        // The blank line before this fn is already in result.
                        // Push the method, then a blank line to separate it
                        // from this fn.
                        result.push(method_fn.clone());
                        result.push(method_body.clone());
                        result.push(method_close.clone());
                        result.push(String::new());
                        method_inserted = true;
                    }
                }
            }

            impl_depth += brace_delta;

            // Closing the impl block (depth went to 0)
            if impl_depth == 0 {
                if !method_inserted {
                    // Insert before closing brace with blank line separator.
                    result.push(String::new());
                    result.push(method_fn.clone());
                    result.push(method_body.clone());
                    result.push(method_close.clone());
                    method_inserted = true;
                }
                in_impl = false;
            }

            result.push(line.to_string());
            continue;
        }

        result.push(line.to_string());
    }

    separate_members(&mut result, |line| {
        line.starts_with("impl Dependencies for")
    });
    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

/// Adds the crate's error variant, its `Display` and `source` arms, and its
/// `From` impl to the `tyt` binary's `Error`.
fn wire_tyt_error(deps: &impl Dependencies, root: &Path, command: &str, name: &str) -> Result<()> {
    let path = root.join(TYT_PROJECT_DIR).join("tyt/src/error.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = kebab_to_snake_case(command);
    let use_line = format!("use tyt_{snake}::Error as {name}Error;");
    let variant_line = format!("    {name}({name}Error),");
    let display_arm = format!("            Error::{name}(e) => e.fmt(f),");
    let source_arm = format!("            Error::{name}(e) => Some(e),");
    let from_sort_key = format!("{name}Error");

    let mut use_inserted = false;
    let mut variant_inserted = false;
    let mut display_inserted = false;
    let mut source_inserted = false;
    let mut from_inserted = false;

    let mut in_enum = false;
    let mut in_display_match = false;
    let mut in_source_match = false;

    for line in &lines {
        let trimmed = line.trim();

        // Insert use in sorted position
        if !use_inserted && trimmed.starts_with("use tyt_") {
            if trimmed > use_line.trim() {
                result.push(use_line.clone());
                use_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }
        if !use_inserted
            && !trimmed.starts_with("use ")
            && result.iter().any(|l| l.starts_with("use "))
        {
            result.push(use_line.clone());
            use_inserted = true;
        }

        // Enum variants
        if trimmed.starts_with("pub enum Error") {
            in_enum = true;
            result.push(line.to_string());
            continue;
        }
        if in_enum {
            if trimmed == "}" {
                if !variant_inserted {
                    result.push(variant_line.clone());
                    variant_inserted = true;
                }
                in_enum = false;
            } else if !variant_inserted && !trimmed.is_empty() && trimmed > variant_line.trim() {
                result.push(variant_line.clone());
                variant_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }

        // Display match arms
        if trimmed.contains("fn fmt(") && trimmed.contains("fmt::") {
            in_display_match = true;
            result.push(line.to_string());
            continue;
        }
        if in_display_match && trimmed.starts_with("Error::") {
            if !display_inserted && trimmed > display_arm.trim() {
                result.push(display_arm.clone());
                display_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }
        if in_display_match && trimmed == "}" {
            if !display_inserted {
                result.push(display_arm.clone());
                display_inserted = true;
            }
            in_display_match = false;
            result.push(line.to_string());
            continue;
        }

        // Source match arms
        if trimmed.contains("fn source(") {
            in_source_match = true;
            result.push(line.to_string());
            continue;
        }
        if in_source_match && trimmed.starts_with("Error::") {
            if !source_inserted && trimmed > source_arm.trim() {
                result.push(source_arm.clone());
                source_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }
        if in_source_match && trimmed == "}" {
            if !source_inserted {
                result.push(source_arm.clone());
                source_inserted = true;
            }
            in_source_match = false;
            result.push(line.to_string());
            continue;
        }

        // From impls, inserted in sorted position
        if !from_inserted
            && trimmed.starts_with("impl From<")
            && trimmed.contains("> for Error")
            && let (Some(start), Some(end)) = (trimmed.find('<'), trimmed.find('>'))
        {
            let from_type = &trimmed[start + 1..end];
            if from_type > from_sort_key.as_str() {
                push_from_impl(&mut result, name, false);
                result.push(String::new());
                from_inserted = true;
            }
        }

        result.push(line.to_string());
    }

    if !from_inserted {
        push_from_impl(&mut result, name, true);
    }

    separate_members(&mut result, |line| line.starts_with("pub enum Error"));

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

/// Pushes the `From<{name}Error>` impl for the `tyt` binary's `Error`.
fn push_from_impl(result: &mut Vec<String>, name: &str, leading_blank: bool) {
    if leading_blank {
        result.push(String::new());
    }
    result.push(format!("impl From<{name}Error> for Error {{"));
    result.push(format!("    fn from(e: {name}Error) -> Self {{"));
    result.push(format!("        Error::{name}(e)"));
    result.push("    }".to_string());
    result.push("}".to_string());
}

/// Adds the crate's subcommand variant and dispatch arm to the `tyt` binary's
/// root `Tyt` enum.
fn wire_tyt_tyt_rs(deps: &impl Dependencies, root: &Path, command: &str, name: &str) -> Result<()> {
    let path = root.join(TYT_PROJECT_DIR).join("tyt/src/tyt.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = kebab_to_snake_case(command);
    let use_line = format!("use tyt_{snake}::Tyt{name};");
    let variant_block = format!(
        "    #[command(name = \"{command}\")]\n    {name} {{\n        #[clap(subcommand)]\n        {snake}: Tyt{name},\n    }},"
    );
    let match_arm = format!(
        "            Tyt::{name} {{ {snake} }} => {snake}.execute(deps.tyt_{snake}_dependencies())?,",
    );

    let mut use_inserted = false;
    let mut variant_inserted = false;
    let mut arm_inserted = false;
    let mut in_enum = false;
    let mut in_match = false;
    let mut enum_depth: i32 = 0;
    let mut match_depth: i32 = 0;
    let mut pending_attrs: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        let brace_delta = trimmed.chars().filter(|&c| c == '{').count() as i32
            - trimmed.chars().filter(|&c| c == '}').count() as i32;

        // Use line insertion
        if !use_inserted && trimmed.starts_with("use tyt_") {
            if trimmed > use_line.trim() {
                result.push(use_line.clone());
                use_inserted = true;
            }
            result.push(line.to_string());
            continue;
        }
        if !use_inserted
            && !trimmed.starts_with("use ")
            && result.iter().any(|l| l.starts_with("use "))
        {
            result.push(use_line.clone());
            use_inserted = true;
        }

        // Enum variants, using brace depth to skip inside variant struct bodies
        if trimmed.starts_with("pub enum Tyt") {
            in_enum = true;
            enum_depth = brace_delta;
            result.push(line.to_string());
            continue;
        }
        if in_enum {
            if enum_depth == 1 && !variant_inserted {
                // Buffer attribute lines so we can insert before them
                if !trimmed.is_empty() && trimmed.starts_with('#') {
                    pending_attrs.push(line.to_string());
                    enum_depth += brace_delta;
                    continue;
                }

                // Compare variant names (skip empty lines and closing brace)
                if !trimmed.is_empty() && trimmed != "}" {
                    let variant_name = trimmed
                        .split(['{', ',', '('])
                        .next()
                        .unwrap_or(trimmed)
                        .trim();
                    if variant_name > name {
                        result.push(variant_block.clone());
                        result.push(String::new());
                        variant_inserted = true;
                    }
                }

                // Flush buffered attributes
                for attr in pending_attrs.drain(..) {
                    result.push(attr);
                }
            }

            enum_depth += brace_delta;

            if enum_depth == 0 {
                if !variant_inserted {
                    for attr in pending_attrs.drain(..) {
                        result.push(attr);
                    }
                    result.push(String::new());
                    result.push(variant_block.clone());
                    variant_inserted = true;
                }
                in_enum = false;
            }

            result.push(line.to_string());
            continue;
        }

        // Match arms, all single-line with balanced braces
        if trimmed.starts_with("match self") {
            in_match = true;
            match_depth = brace_delta;
            result.push(line.to_string());
            continue;
        }
        if in_match {
            if match_depth == 1 && !arm_inserted && trimmed.starts_with("Tyt::") {
                let arm_variant = trimmed
                    .split("::")
                    .nth(1)
                    .unwrap_or("")
                    .split(|c: char| c == '{' || c == '(' || c.is_whitespace())
                    .next()
                    .unwrap_or("");
                if arm_variant > name {
                    result.push(match_arm.clone());
                    arm_inserted = true;
                }
            }

            match_depth += brace_delta;

            if match_depth == 0 {
                if !arm_inserted {
                    result.push(match_arm.clone());
                    arm_inserted = true;
                }
                in_match = false;
            }

            result.push(line.to_string());
            continue;
        }

        result.push(line.to_string());
    }

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}
