use crate::{Dependencies, Result, commands::create_command};
use std::path::Path;

pub fn wire_tyt_error(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root.join("projects/tyt/src/error.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = create_command::kebab_to_snake_case(command);
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
