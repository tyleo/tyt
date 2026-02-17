use crate::Result;

pub fn insert_enum_variant(contents: &str, name: &str, command: &str) -> Result<String> {
    let lines: Vec<&str> = contents.lines().collect();
    let mut result = Vec::new();
    let snake_command = command.replace('-', "_");

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

    // 2. Add enum variant and match arm
    let result_lines: Vec<&str> = result.iter().map(|s| s.as_str()).collect();

    // Extract the execute parameter name from the function signature
    let deps_param = result_lines
        .iter()
        .find_map(|l| {
            let t = l.trim();
            t.strip_prefix("pub fn execute(self,").map(|after| {
                after
                    .split(':')
                    .next()
                    .unwrap_or("dependencies")
                    .trim()
                    .to_string()
            })
        })
        .unwrap_or_else(|| "dependencies".to_string());

    let mut output = String::new();
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
            enum_name = trimmed
                .trim_start_matches("pub enum ")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();

            // Handle empty enum on one line: `pub enum Foo {}`
            if trimmed.ends_with("{}") {
                let prefix = &line[..line.rfind("{}").unwrap()];
                output.push_str(prefix);
                output.push_str("{\n");
                output.push_str(&format!(
                    "    #[command(name = \"{command}\")]\n    {name}({name}),\n"
                ));
                output.push_str("}\n");
                enum_variant_added = true;
                j += 1;
                continue;
            }

            in_enum = true;
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
            // Collect all variant blocks (attribute + variant line), sort, insert
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
        if trimmed.starts_with("match self") {
            // Handle empty match on one line: `match self {}`
            if trimmed.ends_with("{}") {
                let indent = &line[..line.len() - line.trim_start().len()];
                let prefix = &line[..line.rfind("{}").unwrap()];
                output.push_str(prefix);
                output.push_str("{\n");
                output.push_str(&format!(
                    "{indent}    {enum_name}::{name}({snake_command}) => {snake_command}.execute({deps_param}),\n"
                ));
                output.push_str(indent);
                output.push_str("}\n");
                match_arm_added = true;
                j += 1;
                continue;
            }

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
                let arm = format!(
                    "            {enum_name}::{name}({snake_command}) => {snake_command}.execute({deps_param}),"
                );
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
                format!(
                    "{indent}{enum_name}::{name}({snake_command}) => {snake_command}.execute({deps_param}),"
                ),
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
