use crate::{AgentSkill, Dependencies, Result};
use clap::Parser;

/// Prints the skill as a `SKILL.md`.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "print",
    after_help = "`install` writes the skill where an agent loads it."
)]
pub struct IntegrationSkillPrint {}

impl IntegrationSkillPrint {
    /// Runs the command on `skill`.
    pub fn execute(self, skill: AgentSkill, dependencies: impl Dependencies) -> Result<()> {
        Ok(dependencies.write_stdout(skill.skill_md().as_bytes())?)
    }
}
