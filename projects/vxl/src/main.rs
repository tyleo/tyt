use clap::{CommandFactory, Parser, Subcommand};
use std::process;
use ty_clap::{Completion, DependenciesImpl as ClapDependenciesImpl};
use vxl::{
    DependenciesImpl, Error, Vxl,
    commands::{IntegrationAgentsLink, IntegrationSkill},
};

/// Works with voxels.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Sets up other tools to work with vxl.
    #[command(name = "integration", subcommand)]
    Integration(Integration),

    #[command(flatten)]
    Vxl(Box<Vxl>),
}

/// The commands that set up other tools to work with vxl.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
enum Integration {
    /// Links `.claude` to `.agents` for Claude Code.
    #[command(name = "agents-link")]
    AgentsLink(IntegrationAgentsLink),

    /// Prints and installs shell completions.
    #[command(name = "completion", subcommand)]
    Completion(Completion),

    /// Lists, prints, and installs agent skills.
    #[command(name = "skill")]
    Skill(IntegrationSkill),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Integration(Integration::Completion(completion)) => completion
            .execute(&ClapDependenciesImpl, Cli::command(), "vxl")
            .map_err(Error::IO),

        Command::Integration(Integration::AgentsLink(agents_link)) => {
            agents_link.execute(DependenciesImpl)
        }

        Command::Integration(Integration::Skill(skill)) => skill.execute(DependenciesImpl),

        Command::Vxl(cmd) => cmd.execute(DependenciesImpl),
    };

    if let Err(e) = result {
        match e {
            Error::Usage(clap_error) => clap_error.exit(),

            e => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Cli;
    use clap::{Parser, error::ErrorKind};

    #[test]
    fn version_prints_the_crate_version() {
        let error = Cli::try_parse_from(["vxl", "--version"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayVersion);
        assert_eq!(
            error.to_string(),
            format!("vxl {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}
