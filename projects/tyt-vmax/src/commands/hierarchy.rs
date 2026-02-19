use crate::{Dependencies, Result};
use clap::Parser;
use std::path::PathBuf;
use vmax::VMaxScene;
use vmax_serde::VMaxSceneSerde;

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
        let scene_serde: VMaxSceneSerde = serde_json::from_slice(&bytes)?;
        let mut scene: VMaxScene = scene_serde.into();

        scene.objects.sort_by(|a, b| a.name.cmp(&b.name));

        let mut output = String::new();
        let len = scene.objects.len();
        for (i, object) in scene.objects.iter().enumerate() {
            let connector = if i + 1 < len { "\u{251C}" } else { "\u{2514}" };
            output.push_str(&format!("{connector} {}\n", object.name));
        }

        dependencies.write_stdout(output.as_bytes())?;
        Ok(())
    }
}
