use crate::{
    AgentSkill, Dependencies, Result,
    commands::{IntegrationSkillList, IntegrationSkillVerb},
};
use clap::Subcommand;

/// The `integration skill` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum IntegrationSkillCommand {
    #[command(name = "list")]
    IntegrationSkillList(IntegrationSkillList),

    /// Builds a voxel model from a prompt through `vxl sdf-doc`.
    #[command(name = "vxl-model", subcommand)]
    VxlModel(IntegrationSkillVerb),
}

impl IntegrationSkillCommand {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            IntegrationSkillCommand::IntegrationSkillList(list) => list.execute(dependencies),

            IntegrationSkillCommand::VxlModel(verb) => {
                verb.execute(AgentSkill::VxlModel, dependencies)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AgentSkill, commands::IntegrationSkillCommand};
    use clap::{Command, Subcommand, ValueEnum};

    #[test]
    fn every_skill_has_a_subcommand_of_its_name() {
        let group = IntegrationSkillCommand::augment_subcommands(Command::new("skill"));

        let names: Vec<_> = group
            .get_subcommands()
            .map(|command| command.get_name())
            .filter(|name| *name != "list")
            .collect();

        let skills: Vec<_> = AgentSkill::value_variants()
            .iter()
            .map(|skill| skill.name())
            .collect();
        assert_eq!(names, skills);
    }
}
