use crate::{Dependencies, Result};
use clap::Parser;
use std::{collections::HashMap, path::PathBuf};
use vmax::VMaxScene;
use vmax_serde::VMaxSceneSerde;

/// Renames nodes in the Voxel Max scene hierarchy matching a glob pattern.
#[derive(Clone, Debug, Parser)]
#[command(name = "rename-node")]
pub struct RenameNode {
    /// The input `.vmax` directory.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Glob pattern to match hierarchy paths against.
    #[arg(value_name = "pattern")]
    pattern: String,

    /// The new name to assign to matched nodes.
    #[arg(value_name = "new-name")]
    new_name: String,
}

impl RenameNode {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scene_path = self.input_vmax.join("scene.json");
        let bytes = dependencies.read_file(&scene_path)?;

        // Parse as typed scene for hierarchy path building.
        let scene_serde: VMaxSceneSerde = serde_json::from_slice(&bytes)?;
        let scene: VMaxScene = scene_serde.into();

        // Parse as Value for lossless round-trip mutation.
        let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;

        // Build a map from group id -> (name, parent_id) for path construction.
        let group_info: HashMap<&str, (&str, Option<&str>)> = scene
            .groups
            .iter()
            .map(|g| (g.id.as_str(), (g.name.as_str(), g.parent_id.as_deref())))
            .collect();

        // Build full hierarchy path for a node given its name and parent_id.
        let build_path = |name: &str, parent_id: Option<&str>| -> String {
            let mut segments = vec![name];
            let mut current = parent_id;
            while let Some(pid) = current {
                if let Some(&(pname, ppid)) = group_info.get(pid) {
                    segments.push(pname);
                    current = ppid;
                } else {
                    break;
                }
            }
            segments.reverse();
            segments.join("/")
        };

        // Compile glob pattern — auto-prepend `**/` unless already present.
        let pattern = if self.pattern.starts_with("**/") {
            self.pattern.clone()
        } else {
            format!("**/{}", self.pattern)
        };
        let glob = globset::Glob::new(&pattern)?.compile_matcher();

        // Collect matches: (node_path, id, is_group).
        let mut matches: Vec<(String, &str, bool)> = Vec::new();

        for group in &scene.groups {
            let path = build_path(&group.name, group.parent_id.as_deref());
            if glob.is_match(&path) {
                matches.push((path, &group.id, true));
            }
        }

        for object in &scene.objects {
            let path = build_path(&object.name, object.parent_id.as_deref());
            if glob.is_match(&path) {
                matches.push((path, &object.id, false));
            }
        }

        // Apply renames to the serde_json::Value.
        if let Some(groups) = value.get_mut("groups").and_then(|v| v.as_array_mut()) {
            for group_val in groups {
                if let Some(id) = group_val.get("id").and_then(|v| v.as_str()) {
                    if matches
                        .iter()
                        .any(|(_, mid, is_group)| *is_group && *mid == id)
                    {
                        group_val["name"] = serde_json::Value::String(self.new_name.clone());
                    }
                }
            }
        }

        if let Some(objects) = value.get_mut("objects").and_then(|v| v.as_array_mut()) {
            for object_val in objects {
                if let Some(id) = object_val.get("id").and_then(|v| v.as_str()) {
                    if matches
                        .iter()
                        .any(|(_, mid, is_group)| !*is_group && *mid == id)
                    {
                        object_val["n"] = serde_json::Value::String(self.new_name.clone());
                    }
                }
            }
        }

        // Write back.
        let output = serde_json::to_vec_pretty(&value)?;
        dependencies.write_file(&scene_path, &output)?;

        // Print renames.
        let mut stdout_buf = String::new();
        for (path, _, _) in &matches {
            stdout_buf.push_str(&format!("Renamed: {path} -> {}\n", self.new_name));
        }
        dependencies.write_stdout(stdout_buf.as_bytes())?;

        Ok(())
    }
}
