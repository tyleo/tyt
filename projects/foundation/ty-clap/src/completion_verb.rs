use crate::{Dependencies, install_completion, print_completion};
use clap::{Command, Subcommand};
use clap_complete::Shell;
use std::io::Result as IOResult;

const INSTALL_HELP: &str = "\
The completions go under the home directory:
  bash        $XDG_DATA_HOME/bash-completion/completions/<bin>
  elvish      $XDG_DATA_HOME/elvish/completions/<bin>.elv
  fish        $XDG_CONFIG_HOME/fish/completions/<bin>.fish
  powershell  $XDG_DATA_HOME/powershell/completions/<bin>.ps1
  zsh         ~/.zsh/completions/_<bin>

`XDG_DATA_HOME` defaults to `~/.local/share` and `XDG_CONFIG_HOME` to
`~/.config`. A new Bash or Fish shell loads the file. Bash needs
bash-completion 2. For Elvish, PowerShell, and Zsh, the install prints a line
to add to the shell's startup file once. Installing again replaces the file.";

/// The commands on one shell's completions.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum CompletionVerb {
    /// Installs the completions where the shell loads them.
    #[command(name = "install", after_help = INSTALL_HELP)]
    Install,

    /// Prints the completions.
    #[command(
        name = "print",
        after_help = "`install` writes them where the shell loads them."
    )]
    Print,
}

impl CompletionVerb {
    /// Runs the command on `shell`'s completions for the binary `bin`, which
    /// `command` describes.
    pub fn execute(
        self,
        dependencies: &impl Dependencies,
        command: Command,
        bin: &str,
        shell: Shell,
    ) -> IOResult<()> {
        match self {
            CompletionVerb::Install => install_completion(dependencies, command, bin, shell),
            CompletionVerb::Print => print_completion(dependencies, command, bin, shell),
        }
    }
}
