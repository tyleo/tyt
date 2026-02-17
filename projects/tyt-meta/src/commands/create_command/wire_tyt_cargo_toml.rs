use crate::{Dependencies, Result};
use std::path::Path;

pub fn wire_tyt_cargo_toml(deps: &impl Dependencies, root: &Path, command: &str) -> Result<()> {
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
