use crate::{Dependencies, Result};
use std::path::Path;

pub fn wire_workspace_cargo_toml(
    deps: &impl Dependencies,
    root: &Path,
    command: &str,
) -> Result<()> {
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
