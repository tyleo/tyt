use crate::{Dependencies, completion_script};
use clap::Command;
use clap_complete::Shell;
use std::io::Result as IOResult;

/// Writes `shell`'s completions for the binary `bin` to standard output.
pub fn print_completion(
    dependencies: &impl Dependencies,
    mut command: Command,
    bin: &str,
    shell: Shell,
) -> IOResult<()> {
    dependencies.write_stdout(&completion_script(&mut command, bin, shell))
}
