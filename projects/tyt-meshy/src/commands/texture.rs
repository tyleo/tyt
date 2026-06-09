use crate::{Dependencies, Result};
use clap::Parser;

/// Retextures an existing Meshy mesh using the Meshy [Retexture](https://docs.meshy.ai/en/api/retexture) API.
#[derive(Clone, Debug, Parser)]
#[command(name = "texture")]
pub struct Texture {}

impl Texture {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from texture!\n")?;
        Ok(())
    }
}
