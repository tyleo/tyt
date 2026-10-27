use crate::{
    AgentSkill, Dependencies, Result,
    commands::{IntegrationSkillInstall, IntegrationSkillPrint},
};
use clap::Subcommand;

/// The commands on one skill.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum IntegrationSkillVerb {
    #[command(name = "install")]
    IntegrationSkillInstall(IntegrationSkillInstall),

    #[command(name = "print")]
    IntegrationSkillPrint(IntegrationSkillPrint),
}

impl IntegrationSkillVerb {
    /// Runs the command on `skill`.
    pub fn execute(self, skill: AgentSkill, dependencies: impl Dependencies) -> Result<()> {
        match self {
            IntegrationSkillVerb::IntegrationSkillInstall(install) => {
                install.execute(skill, dependencies)
            }

            IntegrationSkillVerb::IntegrationSkillPrint(print) => {
                print.execute(skill, dependencies)
            }
        }
    }
}
