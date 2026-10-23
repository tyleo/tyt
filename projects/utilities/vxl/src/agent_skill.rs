use clap::ValueEnum;
use sdfj_builder::SDFJ_BUILDER_TYPESCRIPT_API;

/// A skill vxl prints as a `SKILL.md` for an agent to load.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AgentSkill {
    /// Builds a voxel model from a prompt through `vxl sdf-doc`.
    #[value(name = "vxl-model")]
    VxlModel,
}

impl AgentSkill {
    /// The skill's name, used as its directory and its command-line value.
    pub fn name(self) -> String {
        self.to_possible_value()
            .expect("every skill has a command-line value")
            .get_name()
            .to_owned()
    }

    /// The skill's `SKILL.md`. The frontmatter records the vxl version that
    /// printed the skill.
    pub fn skill_md(self) -> String {
        let (description, sections) = match self {
            AgentSkill::VxlModel => (
                "Builds a voxel model from a prompt with vxl. Writes a TypeScript \
                 model file of shapes and materials in meters, voxelizes the model, \
                 renders review views, and revises the file until the model matches \
                 the prompt. Exports the model as a glTF mesh when asked. Use when \
                 asked to make, model, or build a voxel model, prop, or scene, or an \
                 .sdfj, .voxj, glTF, or .glb file of one.",
                [
                    include_str!("../skills/vxl-model/workflow.md"),
                    SDFJ_BUILDER_TYPESCRIPT_API,
                    include_str!("../docs/modeling-api.md"),
                ],
            ),
        };
        let version = env!("CARGO_PKG_VERSION");
        let body = sections.join("\n");
        format!(
            "---\nname: {}\ndescription: {description}\nmetadata:\n  vxl-version: \"{version}\"\n\
             ---\n\n{body}",
            self.name(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::AgentSkill;
    use clap::ValueEnum;
    use sdfj_builder::SDFJ_BUILDER_TYPESCRIPT_API;
    use std::collections::HashSet;

    /// The anchor GitHub gives a heading reading `text`.
    fn anchor(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            .map(|c| if c == ' ' { '-' } else { c })
            .collect()
    }

    fn heading(line: &str) -> Option<&str> {
        let text = line.trim_start_matches('#');
        (text.len() < line.len() && text.starts_with(' ')).then(|| text.trim())
    }

    /// The lines of `markdown` outside its fenced code blocks.
    fn prose_lines(markdown: &str) -> Vec<&str> {
        let mut fenced = false;
        markdown
            .lines()
            .filter(|line| {
                if line.starts_with("```") {
                    fenced = !fenced;
                    return false;
                }
                !fenced
            })
            .collect()
    }

    #[test]
    fn each_skill_opens_with_frontmatter_holding_its_name_and_the_vxl_version() {
        for skill in AgentSkill::value_variants() {
            let skill_md = skill.skill_md();

            let name = skill.to_possible_value().unwrap();
            let frontmatter: Vec<_> = skill_md.lines().take(6).collect();
            assert_eq!(frontmatter[0], "---");
            assert_eq!(frontmatter[1], format!("name: {}", name.get_name()));
            // YAML fails to parse a colon and a space inside a plain scalar.
            let description = frontmatter[2].strip_prefix("description: ").unwrap();
            assert!(!description.contains(": "));
            assert_eq!(frontmatter[3], "metadata:");
            assert_eq!(
                frontmatter[4],
                format!("  vxl-version: \"{}\"", env!("CARGO_PKG_VERSION"))
            );
            assert_eq!(frontmatter[5], "---");
        }
    }

    #[test]
    fn the_vxl_model_skill_holds_the_workflow_the_typescript_api_then_the_modeling_api() {
        let skill_md = AgentSkill::VxlModel.skill_md();

        let headings: Vec<_> = prose_lines(&skill_md)
            .into_iter()
            .filter(|line| line.starts_with("# "))
            .collect();
        assert_eq!(
            headings,
            ["# Voxel modeling", "# TypeScript API", "# Modeling API"]
        );
        assert!(skill_md.contains(SDFJ_BUILDER_TYPESCRIPT_API));
        assert!(skill_md.ends_with(include_str!("../docs/modeling-api.md")));
    }

    #[test]
    fn every_link_in_a_skill_reaches_one_of_its_headings_or_the_web() {
        for skill in AgentSkill::value_variants() {
            let skill_md = skill.skill_md();
            let lines = prose_lines(&skill_md);

            let anchors: Vec<_> = lines
                .iter()
                .filter_map(|line| heading(line))
                .map(anchor)
                .collect();
            assert_eq!(
                anchors.iter().collect::<HashSet<_>>().len(),
                anchors.len(),
                "two headings share an anchor"
            );
            let targets = lines
                .iter()
                .flat_map(|line| line.split("](").skip(1))
                .map(|rest| &rest[..rest.find(')').unwrap()]);
            for target in targets {
                match target.strip_prefix('#') {
                    Some(fragment) => assert!(anchors.iter().any(|a| a == fragment), "{target}"),
                    None => assert!(target.starts_with("https://"), "{target}"),
                }
            }
        }
    }
}
