use crate::{Dependencies, Result};
use clap::Parser;
use std::{collections::HashMap, path::PathBuf};

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
        let nodes = dependencies.scene_nodes(&bytes)?;

        // Build a map from group id -> (name, parent_id) for path construction.
        let group_info: HashMap<&str, (&str, Option<&str>)> = nodes
            .iter()
            .filter(|n| n.is_group)
            .map(|n| (n.id.as_str(), (n.name.as_str(), n.parent_id.as_deref())))
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

        // Build all candidate paths.
        let mut candidates: Vec<(String, &str, bool)> = Vec::new();
        for node in &nodes {
            let path = build_path(&node.name, node.parent_id.as_deref());
            candidates.push((path, &node.id, node.is_group));
        }

        let candidate_paths: Vec<&str> = candidates.iter().map(|(p, _, _)| p.as_str()).collect();
        let matched = dependencies.match_glob(&pattern, &candidate_paths)?;

        // Collect matched group/object IDs.
        let mut group_ids: Vec<&str> = Vec::new();
        let mut object_ids: Vec<&str> = Vec::new();
        let mut renamed: Vec<(&str, bool)> = Vec::new();

        for (i, &is_match) in matched.iter().enumerate() {
            if is_match {
                let (_, id, is_group) = &candidates[i];
                if *is_group {
                    group_ids.push(id);
                } else {
                    object_ids.push(id);
                }
                renamed.push((candidate_paths[i], *is_group));
            }
        }

        // Apply renames via lossless JSON round-trip.
        let output = dependencies.rename_scene_nodes_json(
            &bytes,
            &group_ids,
            &object_ids,
            &self.new_name,
        )?;
        dependencies.write_file(&scene_path, &output)?;

        // Print renames.
        let mut stdout_buf = String::new();
        for (path, _) in &renamed {
            stdout_buf.push_str(&format!("Renamed: {path} -> {}\n", self.new_name));
        }
        dependencies.write_stdout(stdout_buf.as_bytes())?;

        Ok(())
    }
}
