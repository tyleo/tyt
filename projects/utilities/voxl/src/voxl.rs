use crate::{Dependencies, Result};
use clap::Args;

/// A command-line tool for working with voxels.
#[derive(Args, Clone, Debug)]
pub struct Voxl {}

impl Voxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"voxl: not yet implemented\n")
    }
}
