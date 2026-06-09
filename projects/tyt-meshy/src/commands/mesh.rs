use crate::{Dependencies, Result};
use clap::Parser;

/// Generates a 3D mesh from an image using the Meshy [Image to 3D](https://docs.meshy.ai/en/api/image-to-3d) API.
#[derive(Clone, Debug, Parser)]
#[command(name = "mesh")]
pub struct Mesh {}

impl Mesh {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from mesh!\n")?;
        Ok(())
    }
}
