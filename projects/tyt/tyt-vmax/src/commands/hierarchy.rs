use crate::{Dependencies, Result};
use clap::Parser;
use std::{collections::HashMap, path::PathBuf};

/// Prints the Voxel Max hierarchy.
#[derive(Clone, Debug, Parser)]
#[command(name = "hierarchy")]
pub struct Hierarchy {
    /// The input `.vmax` directory to inspect.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scene_path = self.input_vmax.join("scene.json");
        let bytes = dependencies.read_file(&scene_path)?;
        let nodes = dependencies.scene_nodes(&bytes)?;

        // Collect children (name plus, for groups, their ID) keyed by parent ID.
        // None key = root level.
        let mut children: HashMap<Option<&str>, Vec<(&str, Option<&str>)>> = HashMap::new();

        for node in &nodes {
            let group_id = node.is_group.then_some(node.id.as_str());
            children
                .entry(node.parent_id.as_deref())
                .or_default()
                .push((&node.name, group_id));
        }

        // Sort children alphabetically at each level.
        for list in children.values_mut() {
            list.sort_by(|a, b| a.0.cmp(b.0));
        }

        let mut output = String::new();
        print_tree(&children, None, "", &mut output);

        dependencies.write_stdout(output.as_bytes())?;
        Ok(())
    }
}

fn print_tree(
    children: &HashMap<Option<&str>, Vec<(&str, Option<&str>)>>,
    parent_id: Option<&str>,
    prefix: &str,
    output: &mut String,
) {
    let Some(nodes) = children.get(&parent_id) else {
        return;
    };
    let len = nodes.len();
    for (i, &(name, group_id)) in nodes.iter().enumerate() {
        let is_last = i + 1 == len;
        let connector = if is_last { "\u{2514}" } else { "\u{251C}" };
        let kind = if group_id.is_some() {
            "Group"
        } else {
            "Object"
        };
        output.push_str(&format!("{prefix}{connector} {name} ({kind})\n"));

        if let Some(id) = group_id {
            let child_prefix = if is_last {
                format!("{prefix}  ")
            } else {
                format!("{prefix}\u{2502} ")
            };
            print_tree(children, Some(id), &child_prefix, output);
        }
    }
}
