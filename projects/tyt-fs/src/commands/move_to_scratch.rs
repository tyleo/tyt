use crate::{Dependencies, Error, Result, utilities};
use clap::Parser;
use std::path::PathBuf;

/// Moves files to a scratch directory defined in the .tytconfig file.
#[derive(Clone, Debug, Parser)]
#[command(name = "move-to-scratch")]
pub struct MoveToScratch {
    #[arg(value_name = "pattern", required = true)]
    patterns: Vec<String>,
}

impl MoveToScratch {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scratch_dir = dependencies
            .scratch_dir()?
            .ok_or(Error::ScratchDirNotConfigured)?;

        let stdout = utilities::find_files(&dependencies, &self.patterns)?;
        if stdout.is_empty() {
            return Ok(());
        }

        dependencies.create_dir_all(&scratch_dir)?;

        for line in stdout.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            let src = PathBuf::from(String::from_utf8_lossy(line).as_ref());
            let Some(file_name) = src.file_name() else {
                continue;
            };
            let dest = scratch_dir.join(file_name);
            dependencies.rename(&src, &dest)?;
            dependencies.write_stdout(line)?;
            dependencies.write_stdout(b"\n")?;
        }

        Ok(())
    }
}
