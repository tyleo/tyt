use crate::{AgentSkill, Dependencies, Result};
use clap::{Parser, ValueEnum};

/// Lists every skill vxl ships, one row apiece.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct IntegrationSkillList {}

impl IntegrationSkillList {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let values: Vec<_> = AgentSkill::value_variants()
            .iter()
            .map(|skill| {
                skill
                    .to_possible_value()
                    .expect("every skill has a command-line value")
            })
            .collect();

        let width = values
            .iter()
            .map(|value| value.get_name().len())
            .max()
            .expect("vxl ships a skill");

        let rows: String = values
            .iter()
            .map(|value| {
                let help = value.get_help().expect("every skill has a description");

                format!("{:width$}  {help}\n", value.get_name())
            })
            .collect();

        Ok(dependencies.write_stdout(rows.as_bytes())?)
    }
}
