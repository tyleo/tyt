use crate::commands::{AddProfile, CopyProfileSettings, ListProfiles, Run, SetProfile};
use clap::Subcommand;

/// Operations for working with claude
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytClaude {
    #[command(name = "add-profile")]
    AddProfile(AddProfile),
    #[command(name = "copy-profile-settings")]
    CopyProfileSettings(CopyProfileSettings),
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
            TytClaude::AddProfile(add_profile) => add_profile.execute(_dependencies),
            TytClaude::CopyProfileSettings(copy_profile_settings) => {
                copy_profile_settings.execute(_dependencies)
            }
            TytClaude::ListProfiles(list_profiles) => list_profiles.execute(_dependencies),
            TytClaude::Run(run) => run.execute(_dependencies),
            TytClaude::SetProfile(set_profile) => set_profile.execute(_dependencies),
        }
    }
}
