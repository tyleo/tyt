use clap::Command;
use clap_complete::Shell;

/// `shell`'s completion script for the binary `bin`, which `command` describes.
pub fn completion_script(command: &mut Command, bin: &str, shell: Shell) -> Vec<u8> {
    let mut script = Vec::new();

    clap_complete::generate(shell, command, bin, &mut script);

    script
}
