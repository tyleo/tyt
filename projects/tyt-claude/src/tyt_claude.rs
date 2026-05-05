use crate::commands::{CreateProfile, ListProfiles, Run, SetProfile};
use clap::Subcommand;

/// Operations for working with claude
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytClaude {
    #[command(name = "create-profile")]
    CreateProfile(CreateProfile),
    #[command(name = "list-profiles")]
    ListProfiles(ListProfiles),
    #[command(name = "run")]
    Run(Run),
    #[command(name = "set-profile")]
    SetProfile(SetProfile),
}

impl TytClaude {
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {
        match self {
            TytClaude::CreateProfile(create_profile) => create_profile.execute(_dependencies),
            TytClaude::ListProfiles(list_profiles) => list_profiles.execute(_dependencies),
            TytClaude::Run(run) => run.execute(_dependencies),
            TytClaude::SetProfile(set_profile) => set_profile.execute(_dependencies),
        }
    }
}
