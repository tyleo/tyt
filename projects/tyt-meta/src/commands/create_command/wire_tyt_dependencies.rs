use crate::{Dependencies, Result, commands::create_command::snake};
use std::path::Path;

pub fn wire_tyt_dependencies(
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
