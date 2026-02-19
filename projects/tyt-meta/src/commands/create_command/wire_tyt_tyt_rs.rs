use crate::{Dependencies, Result, commands::create_command};
use std::path::Path;

pub fn wire_tyt_tyt_rs(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root.join("projects/tyt/src/tyt.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = create_command::kebab_to_snake_case(command);
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

        // Enum variants — use brace depth to skip inside variant struct bodies
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
