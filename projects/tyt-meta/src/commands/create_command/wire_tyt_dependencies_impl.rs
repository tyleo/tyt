use crate::{Dependencies, Result, commands::create_command};
use std::path::Path;

pub fn wire_tyt_dependencies_impl(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
    name: &str,
) -> Result<()> {
    let path = root.join("projects/tyt/src/dependencies_impl.rs");
    let contents = deps.read_to_string(&path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut result: Vec<String> = Vec::new();

    let snake = create_command::kebab_to_snake_case(command);
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
