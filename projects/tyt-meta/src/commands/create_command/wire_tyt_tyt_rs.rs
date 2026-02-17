use crate::{Dependencies, Result, commands::create_command::kebab_to_snake_case};
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

    let snake = kebab_to_snake_case(command);
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
