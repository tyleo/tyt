use crate::{Dependencies, Result};
use clap::Parser;

/// Generates images using teh OpenAI API, saving conversations to disc.
#[derive(Clone, Debug, Parser)]
#[command(name = "img")]
pub struct Img {}

impl Img {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from img!\n")?;
        Ok(())
    }
}
