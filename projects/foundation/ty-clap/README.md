# ty-clap

Shared [clap](https://crates.io/crates/clap) commands for command-line binaries.

## Completion

`Completion` holds the `integration completion <shell> print|install`
commands. A binary nests it under its `integration` group and runs it with its
command and binary name:

```rust
use clap::{CommandFactory, Parser, Subcommand};
use ty_clap::{Completion, DependenciesImpl};

/// A binary with completions.
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    integration: Integration,
}

#[derive(Subcommand)]
enum Integration {
    #[command(name = "completion", subcommand)]
    Completion(Completion),
}

fn main() {
    let Integration::Completion(completion) = Cli::parse().integration;

    completion
        .execute(&DependenciesImpl, Cli::command(), "demo")
        .unwrap();
}
```

`print` writes a shell's completion script to standard output. `install` writes
it under the home directory where the shell loads it. `XDG_DATA_HOME` and
`XDG_CONFIG_HOME` move it when set. For elvish, PowerShell, and zsh, `install`
also prints the one line the shell's startup file needs.
