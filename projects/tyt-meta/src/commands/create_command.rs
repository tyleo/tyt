use crate::{Dependencies, Error, Result};
use clap::Parser;
use std::path::Path;

/// Scaffolds a new tyt sub-crate or adds a command to an existing one.
///
/// Without `--parent`, creates a brand-new `tyt-{command}` sub-crate with all
/// boilerplate and wires it into the workspace and top-level `tyt` binary.
///
/// With `--parent`, adds a command to an existing sub-crate.
#[derive(Clone, Debug, Parser)]
#[command(name = "create-command")]
pub struct CreateCommand {
    /// PascalCase type name (e.g., `FooBar`).
    #[arg(value_name = "name")]
    name: String,

    /// kebab-case CLI name (e.g., `foo-bar`).
    #[arg(value_name = "command")]
    command: String,

    /// Description for doc comments, Cargo.toml, and README.
    #[arg(value_name = "description")]
    description: String,

    /// Existing crate suffix to add the command to (e.g., `fbx` for `tyt-fbx`).
    #[arg(value_name = "parent", short, long)]
    parent: Option<String>,
}

impl CreateCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match &self.parent {
            Some(parent) => add_command_to_crate(&self, &dependencies, parent),
            None => create_crate(&self, &dependencies),
        }
    }
}

fn snake(command: &str) -> String {
    command.replace('-', "_")
}

// ---------------------------------------------------------------------------
// create_crate: brand-new tyt-{command} sub-crate
// ---------------------------------------------------------------------------

fn create_crate(cmd: &CreateCommand, deps: &impl Dependencies) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let snake = snake(command);
    let root = deps.workspace_root()?;
    let crate_dir = root.join(format!("projects/tyt-{command}"));

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
        &cargo_toml_template(command, description),
    )?;

    // 2. LICENSE
    deps.write(crate_dir.join("LICENSE"), LICENSE_TEMPLATE)?;

    // 3. README.md
    deps.write(
        crate_dir.join("README.md"),
        &readme_template(command, name, description),
    )?;

    // 4. src/lib.rs
    deps.write(src.join("lib.rs"), &lib_rs_template(&snake))?;

    // 5. src/main.rs
    deps.write(src.join("main.rs"), &main_rs_template(command, name))?;

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

    // 10. src/tyt_{snake}.rs
    deps.write(
        src.join(format!("tyt_{snake}.rs")),
        &tyt_enum_template_empty(name, description),
    )?;

    // 11. src/commands/mod.rs
    deps.write(commands_dir.join("mod.rs"), "")?;

    // -- Wire into existing files --

    // Workspace Cargo.toml
    wire_workspace_cargo_toml(deps, &root, command)?;

    // projects/tyt/Cargo.toml
    wire_tyt_cargo_toml(deps, &root, command)?;

    // projects/tyt/src/dependencies.rs
    wire_tyt_dependencies(deps, &root, command, name)?;

    // projects/tyt/src/dependencies_impl.rs
    wire_tyt_dependencies_impl(deps, &root, command, name)?;

    // projects/tyt/src/error.rs
    wire_tyt_error(deps, &root, command, name)?;

    // projects/tyt/src/tyt.rs
    wire_tyt_tyt_rs(deps, &root, command, name)?;

    deps.write_stdout(
        format!(
            "Created tyt-{command} crate and wired into workspace.\n\
             Next: add commands with `tyt-meta create-command <Name> <command> <desc> --parent {command}`\n"
        )
        .as_bytes(),
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// add_command_to_crate: add a command to an existing sub-crate
// ---------------------------------------------------------------------------

fn add_command_to_crate(cmd: &CreateCommand, deps: &impl Dependencies, parent: &str) -> Result<()> {
    let command = &cmd.command;
    let name = &cmd.name;
    let description = &cmd.description;
    let command_snake = snake(command);
    let parent_snake = snake(parent);
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
        &command_file_template(name, command, description),
    )?;

    // 2. Update commands/mod.rs
    let mod_path = parent_dir.join("src/commands/mod.rs");
    let mod_contents = deps.read_to_string(&mod_path)?;
    let new_mod = insert_command_mod(&mod_contents, &command_snake);
    deps.write(&mod_path, &new_mod)?;

    // 3. Update tyt_{parent_snake}.rs
    let enum_path = parent_dir.join(format!("src/tyt_{parent_snake}.rs"));
    let enum_contents = deps.read_to_string(&enum_path)?;
    let new_enum = insert_enum_variant(&enum_contents, name, command)?;
    deps.write(&enum_path, &new_enum)?;

    deps.write_stdout(
        format!("Added `{name}` command (`{command}`) to tyt-{parent}.\n").as_bytes(),
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Template: new command file
// ---------------------------------------------------------------------------

fn command_file_template(name: &str, command: &str, description: &str) -> String {
    format!(
        r#"use crate::{{Dependencies, Result}};
use clap::Parser;

/// {description}
#[derive(Clone, Debug, Parser)]
#[command(name = "{command}")]
pub struct {name} {{}}

impl {name} {{
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {{
        dependencies.write_stdout(b"Hello from {command}!\n")?;
        Ok(())
    }}
}}
"#
    )
}

// ---------------------------------------------------------------------------
// Template: commands/mod.rs insertion
// ---------------------------------------------------------------------------

fn insert_command_mod(contents: &str, command_snake: &str) -> String {
    let mod_line = format!("mod {command_snake};");
    let use_line = format!("pub use {command_snake}::*;");

    let lines: Vec<&str> = contents.lines().collect();

    // Split into mod block and use block
    let mut mod_lines: Vec<String> = Vec::new();
    let mut use_lines: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("mod ") {
            mod_lines.push(line.to_string());
        } else if trimmed.starts_with("pub use ") {
            use_lines.push(line.to_string());
        }
    }

    mod_lines.push(mod_line);
    mod_lines.sort();
    use_lines.push(use_line);
    use_lines.sort();

    let mut result = mod_lines.join("\n");
    result.push_str("\n\n");
    result.push_str(&use_lines.join("\n"));
    result.push('\n');
    result
}

// ---------------------------------------------------------------------------
// Template: enum variant insertion in tyt_{parent}.rs
// ---------------------------------------------------------------------------

fn insert_enum_variant(contents: &str, name: &str, command: &str) -> Result<String> {
    let lines: Vec<&str> = contents.lines().collect();
    let mut result = Vec::new();

    // 1. Add use import — find the `use crate::commands::{...}` line and add the new name
    let mut i = 0;
    let mut use_handled = false;
    while i < lines.len() {
        if !use_handled && lines[i].starts_with("use crate::commands::") {
            // Could be single-line `use crate::commands::Foo;` or multi `use crate::commands::{...};`
            let line = lines[i];
            if line.contains('{') {
                // Multi-import: collect all names, add new one, re-sort
                let mut names = Vec::new();
                // Might be single-line or multi-line braces
                if line.contains('}') {
                    // Single line with braces: `use crate::commands::{A, B};`
                    let start = line.find('{').unwrap() + 1;
                    let end = line.find('}').unwrap();
                    for n in line[start..end].split(',') {
                        let n = n.trim();
                        if !n.is_empty() {
                            names.push(n.to_string());
                        }
                    }
                    names.push(name.to_string());
                    names.sort();
                    result.push(format!("use crate::commands::{{{}}};", names.join(", ")));
                    i += 1;
                } else {
                    // Multi-line braces
                    i += 1;
                    while i < lines.len() && !lines[i].contains('}') {
                        let n = lines[i].trim().trim_end_matches(',');
                        if !n.is_empty() {
                            names.push(n.to_string());
                        }
                        i += 1;
                    }
                    names.push(name.to_string());
                    names.sort();
                    result.push(format!("use crate::commands::{{{}}};", names.join(", ")));
                    i += 1; // skip closing brace line
                }
            } else {
                // Single import without braces: `use crate::commands::Foo;`
                let existing = line
                    .trim_start_matches("use crate::commands::")
                    .trim_end_matches(';')
                    .to_string();
                let mut names = [existing, name.to_string()];
                names.sort();
                result.push(format!("use crate::commands::{{{}}};", names.join(", ")));
                i += 1;
            }
            use_handled = true;
            continue;
        }
        result.push(lines[i].to_string());
        i += 1;
    }

    // If no existing use crate::commands line, add one before the first use line
    if !use_handled {
        let insert_pos = result
            .iter()
            .position(|l| l.starts_with("use "))
            .unwrap_or(0);
        result.insert(insert_pos, format!("use crate::commands::{name};"));
    }

    // 2. Add enum variant — find the closing `}` of the enum and insert before it
    let mut output = String::new();
    let result_lines: Vec<&str> = result.iter().map(|s| s.as_str()).collect();
    let mut j = 0;
    let mut in_enum = false;
    let mut enum_variant_added = false;
    let mut in_match = false;
    let mut match_arm_added = false;
    let mut enum_name = String::new();

    while j < result_lines.len() {
        let line = result_lines[j];
        let trimmed = line.trim();

        // Detect enum declaration
        if trimmed.starts_with("pub enum ") {
            in_enum = true;
            enum_name = trimmed
                .trim_start_matches("pub enum ")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            output.push_str(line);
            output.push('\n');
            j += 1;
            continue;
        }

        // Inside enum body: insert variant in sorted position
        if in_enum && !enum_variant_added {
            if trimmed == "}" {
                // Empty enum or end — insert before closing brace
                let variant_line =
                    format!("    #[command(name = \"{command}\")]\n    {name}({name}),");
                output.push_str(&variant_line);
                output.push('\n');
                enum_variant_added = true;
                in_enum = false;
                output.push_str(line);
                output.push('\n');
                j += 1;
                continue;
            }
            // Check if this line is a variant or attribute
            // Collect variants to sort
            // Actually, let's collect all variant blocks (attribute + variant line), sort, insert
            let mut variant_blocks: Vec<(String, String)> = Vec::new(); // (sort_key, block_text)
            let mut current_block = String::new();
            while j < result_lines.len() && result_lines[j].trim() != "}" {
                let l = result_lines[j];
                let t = l.trim();
                if t.starts_with("#[") {
                    current_block.push_str(l);
                    current_block.push('\n');
                } else if !t.is_empty() {
                    // This is the variant line
                    let sort_key = t
                        .split('(')
                        .next()
                        .unwrap_or(t)
                        .split('{')
                        .next()
                        .unwrap_or(t)
                        .trim()
                        .to_string();
                    current_block.push_str(l);
                    current_block.push('\n');
                    variant_blocks.push((sort_key, current_block.clone()));
                    current_block = String::new();
                }
                j += 1;
            }
            // Add new variant block
            let new_block = format!("    #[command(name = \"{command}\")]\n    {name}({name}),\n");
            variant_blocks.push((name.to_string(), new_block));
            variant_blocks.sort_by(|a, b| a.0.cmp(&b.0));

            for (_, block) in &variant_blocks {
                output.push_str(block);
            }
            enum_variant_added = true;
            in_enum = false;
            // j now points at "}"
            output.push_str(result_lines[j]);
            output.push('\n');
            j += 1;
            continue;
        }

        // Detect match self
        if trimmed.starts_with("match self") || trimmed.starts_with("match self {") {
            in_match = true;
            output.push_str(line);
            output.push('\n');
            j += 1;
            continue;
        }

        // Inside match body: insert arm in sorted position
        if in_match && !match_arm_added {
            if trimmed == "}" {
                // Insert before closing brace
                let arm =
                    format!("            {enum_name}::{name}(cmd) => cmd.execute(dependencies),");
                output.push_str(&arm);
                output.push('\n');
                match_arm_added = true;
                in_match = false;
                output.push_str(line);
                output.push('\n');
                j += 1;
                continue;
            }
            // Collect match arms, sort, add new one
            let mut arms: Vec<(String, String)> = Vec::new();
            while j < result_lines.len() && result_lines[j].trim() != "}" {
                let l = result_lines[j];
                let t = l.trim();
                if !t.is_empty() {
                    let sort_key = t
                        .split("::")
                        .nth(1)
                        .unwrap_or(t)
                        .split('(')
                        .next()
                        .unwrap_or(t)
                        .to_string();
                    arms.push((sort_key, l.to_string()));
                }
                j += 1;
            }
            let indent = if arms.is_empty() {
                "            ".to_string()
            } else {
                let first = &arms[0].1;
                first[..first.len() - first.trim_start().len()].to_string()
            };
            arms.push((
                name.to_string(),
                format!("{indent}{enum_name}::{name}(cmd) => cmd.execute(dependencies),"),
            ));
            arms.sort_by(|a, b| a.0.cmp(&b.0));
            for (_, arm) in &arms {
                output.push_str(arm);
                output.push('\n');
            }
            match_arm_added = true;
            in_match = false;
            // j points at "}"
            output.push_str(result_lines[j]);
            output.push('\n');
            j += 1;
            continue;
        }

        output.push_str(line);
        output.push('\n');
        j += 1;
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Wire: workspace Cargo.toml
// ---------------------------------------------------------------------------

fn wire_workspace_cargo_toml(deps: &impl Dependencies, root: &Path, command: &str) -> Result<()> {
    let path = root.join("Cargo.toml");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let member_entry = format!("    \"projects/tyt-{command}\",");
    let patch_entry = format!("tyt-{command} = {{ path = \"projects/tyt-{command}\" }}");

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

// ---------------------------------------------------------------------------
// Wire: projects/tyt/Cargo.toml
// ---------------------------------------------------------------------------

fn wire_tyt_cargo_toml(deps: &impl Dependencies, root: &Path, command: &str) -> Result<()> {
    let path = root.join("projects/tyt/Cargo.toml");
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

// ---------------------------------------------------------------------------
// Wire: projects/tyt/src/dependencies.rs
// ---------------------------------------------------------------------------

fn wire_tyt_dependencies(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root.join("projects/tyt/src/dependencies.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = snake(command);
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
            // Past all use lines — insert at end of use block
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

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

// ---------------------------------------------------------------------------
// Wire: projects/tyt/src/dependencies_impl.rs
// ---------------------------------------------------------------------------

fn wire_tyt_dependencies_impl(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root.join("projects/tyt/src/dependencies_impl.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = snake(command);
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
            && !trimmed.is_empty()
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

                if !type_inserted && trimmed.is_empty() {
                    result.push(type_line.clone());
                    type_inserted = true;
                    past_types = true;
                    result.push(line.to_string());
                    impl_depth += brace_delta;
                    continue;
                }

                if type_inserted && !past_types && trimmed.is_empty() {
                    past_types = true;
                }

                if past_types && !method_inserted && trimmed.starts_with("fn ") {
                    let method_sort_key = format!("fn tyt_{snake}_dependencies");
                    if trimmed > method_sort_key.as_str() {
                        // The blank line before this fn is already in result.
                        // Push method, then blank line to separate from this fn.
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

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

// ---------------------------------------------------------------------------
// Wire: projects/tyt/src/error.rs
// ---------------------------------------------------------------------------

fn wire_tyt_error(deps: &impl Dependencies, root: &Path, command: &str, name: &str) -> Result<()> {
    let path = root.join("projects/tyt/src/error.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = snake(command);
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
            && !trimmed.is_empty()
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
            in_source_match = false;
            result.push(line.to_string());
            continue;
        }

        // From impls — insert in sorted position
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

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    deps.write(&path, &output)
}

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

// ---------------------------------------------------------------------------
// Wire: projects/tyt/src/tyt.rs
// ---------------------------------------------------------------------------

fn wire_tyt_tyt_rs(deps: &impl Dependencies, root: &Path, command: &str, name: &str) -> Result<()> {
    let path = root.join("projects/tyt/src/tyt.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = snake(command);
    let use_line = format!("use tyt_{snake}::Tyt{name};");
    let variant_block =
        format!("    {name} {{\n        #[clap(subcommand)]\n        {snake}: Tyt{name},\n    }},");
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
            && !trimmed.is_empty()
            && result.iter().any(|l| l.starts_with("use "))
        {
            result.push(use_line.clone());
            use_inserted = true;
        }

        // Enum variants — use brace depth to skip inside variant struct bodies
        if trimmed.starts_with("pub enum Tyt") {
            in_enum = true;
            enum_depth = brace_delta;
            result.push(line.to_string());
            continue;
        }
        if in_enum {
            // Only compare variant names at enum top-level (depth 1)
            if enum_depth == 1
                && !variant_inserted
                && !trimmed.is_empty()
                && !trimmed.starts_with('#')
            {
                let variant_name = trimmed
                    .split(['{', ',', '('])
                    .next()
                    .unwrap_or(trimmed)
                    .trim();
                if variant_name > name {
                    result.push(variant_block.clone());
                    variant_inserted = true;
                }
            }

            enum_depth += brace_delta;

            if enum_depth == 0 {
                if !variant_inserted {
                    result.push(variant_block.clone());
                    variant_inserted = true;
                }
                in_enum = false;
            }

            result.push(line.to_string());
            continue;
        }

        // Match arms — all arms are single-line with balanced braces
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

// ---------------------------------------------------------------------------
// File content templates
// ---------------------------------------------------------------------------

fn cargo_toml_template(command: &str, description: &str) -> String {
    format!(
        r#"[package]
name = "tyt-{command}"
version = "0.1.0"
edition = "2024"
license-file = "LICENSE"
description = "{description}"

[dependencies]
clap = {{ version = "4.5.58", features = ["derive"] }}
tyt-common = {{ version = "0.1.0" }}
tyt-injection = {{ version = "0.1.0", optional = true }}

[features]
default = ["impl"]
impl = ["dep:tyt-injection"]
"#
    )
}

const LICENSE_TEMPLATE: &str = "\
MIT License

Copyright (c) 2026 tyleo

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
";

fn readme_template(command: &str, name: &str, description: &str) -> String {
    format!("# tyt-{command} - {name}\n\n{description}\n")
}

fn lib_rs_template(snake: &str) -> String {
    format!(
        r#"pub mod commands;

mod dependencies;
#[cfg(feature = "impl")]
mod dependencies_impl;
mod error;
mod result;
mod tyt_{snake};

pub use dependencies::*;
#[cfg(feature = "impl")]
pub use dependencies_impl::*;
pub use error::*;
pub use result::*;
pub use tyt_{snake}::*;
"#
    )
}

fn main_rs_template(command: &str, name: &str) -> String {
    let snake = snake(command);
    format!(
        r#"use clap::Parser;
use tyt_{snake}::{{DependenciesImpl, Tyt{name}}};

#[derive(Clone, Debug, Parser)]
pub struct Cli {{
    #[clap(subcommand)]
    pub command: Tyt{name},
}}

fn main() {{
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {{
        eprintln!("error: {{e}}");
        std::process::exit(1);
    }}
}}
"#
    )
}

const DEPENDENCIES_RS_TEMPLATE: &str = r#"use crate::Result;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
"#;

const DEPENDENCIES_IMPL_RS_TEMPLATE: &str = r#"use crate::{Dependencies, Result};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
"#;

const ERROR_RS_TEMPLATE: &str = r#"use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    IO(IOError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::IO(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
"#;

const RESULT_RS_TEMPLATE: &str = r#"use crate::Error;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;
"#;

fn tyt_enum_template_empty(name: &str, description: &str) -> String {
    format!(
        r#"use clap::Subcommand;

/// {description}
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tyt{name} {{}}

impl Tyt{name} {{
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {{
        match self {{}}
    }}
}}
"#
    )
}
