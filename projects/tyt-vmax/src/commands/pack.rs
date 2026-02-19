use crate::{Dependencies, Result};
use clap::Parser;
use std::path::PathBuf;

const HISTORY_EXTENSIONS: &[&str] = &["vmaxhb", "vmaxhvsb", "vmaxhvsc"];

/// Packs a .vmax directory by stripping history files.
#[derive(Clone, Debug, Parser)]
#[command(name = "pack")]
pub struct Pack {
    /// The input `.vmax` directory to pack.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Optional output `.vmax` directory. If provided, copies the input first.
    #[arg(value_name = "output-vmax", long)]
    output_vmax: Option<PathBuf>,
}

impl Pack {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let target = if let Some(ref output) = self.output_vmax {
            dependencies.copy_dir(&self.input_vmax, output)?;
            output.clone()
        } else {
            self.input_vmax.clone()
        };

        // Update scene.json to clear hist fields.
        let scene_path = target.join("scene.json");
        let scene_bytes = dependencies.read_file(&scene_path)?;
        let packed_bytes = dependencies.pack_scene_json(&scene_bytes)?;
        dependencies.write_file(&scene_path, &packed_bytes)?;

        // Remove history files.
        let entries = dependencies.list_dir(&target)?;
        let mut output_buf = String::new();
        for entry in entries {
            let is_history = entry
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| HISTORY_EXTENSIONS.contains(&ext));
            if is_history {
                dependencies.remove_file(&entry)?;
                output_buf.push_str(&format!("Removed: {}\n", entry.display()));
            }
        }
        dependencies.write_stdout(output_buf.as_bytes())?;

        Ok(())
    }
}
