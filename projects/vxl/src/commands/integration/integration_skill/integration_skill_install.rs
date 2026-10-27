use crate::{
    AgentSkill, Dependencies, Result, WriteFile, WriteStdout,
    commands::{IntegrationRoot, SKILL_FILE, SkillTarget},
};
use clap::Parser;
use std::path::Path;

const INSTALL_HELP: &str = "\
The skill goes in `.<target>/skills/<skill>/SKILL.md` at the git root, or under
the home directory with `--user`. Codex reads `agents` and Claude Code reads
`claude`. `integration agents-link` links `.claude` to `.agents` so both read
one copy.

Installing again upgrades the skill by replacing the `SKILL.md`.";

/// Installs the skill where an agent loads it.
#[derive(Clone, Debug, Parser)]
#[command(name = "install", after_help = INSTALL_HELP)]
pub struct IntegrationSkillInstall {
    /// The skills directory to install into.
    #[arg(value_name = "target")]
    target: SkillTarget,

    #[command(flatten)]
    root: IntegrationRoot,
}

impl IntegrationSkillInstall {
    /// Runs the command on `skill`.
    pub fn execute(self, skill: AgentSkill, dependencies: impl Dependencies) -> Result<()> {
        let root = self.root.resolve(&dependencies)?;

        install_skill(&dependencies, skill, self.target, &root)
    }
}

fn install_skill(
    dependencies: &(impl WriteFile + WriteStdout),
    skill: AgentSkill,
    target: SkillTarget,
    root: &Path,
) -> Result<()> {
    let path = target.skills_dir(root).join(skill.name()).join(SKILL_FILE);

    dependencies.write_file(&path, skill.skill_md().as_bytes())?;

    Ok(dependencies.write_stdout(format!("installed {}\n", path.display()).as_bytes())?)
}

#[cfg(test)]
mod tests {
    use crate::{
        AgentSkill, CapturedStdout,
        commands::{
            SkillTarget, integration::integration_skill::integration_skill_install::install_skill,
        },
    };
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    fn install(root: &TempDir, target: SkillTarget) -> String {
        let dependencies = CapturedStdout::default();

        install_skill(&dependencies, AgentSkill::VxlModel, target, root.path()).unwrap();

        dependencies.stdout()
    }

    fn read(root: &TempDir, path: &str) -> String {
        fs::read_to_string(root.path().join(path)).unwrap()
    }

    #[test]
    fn install_writes_the_skill_under_the_target_and_reports_the_path() {
        for (target, path) in [
            (SkillTarget::Agents, ".agents/skills/vxl-model/SKILL.md"),
            (SkillTarget::Claude, ".claude/skills/vxl-model/SKILL.md"),
        ] {
            let root = TempDir::new().unwrap();

            let stdout = install(&root, target);

            assert_eq!(read(&root, path), AgentSkill::VxlModel.skill_md());
            assert_eq!(
                stdout,
                format!("installed {}\n", root.path().join(path).display())
            );
        }
    }

    #[test]
    fn install_replaces_an_existing_skill_md() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".agents/skills/vxl-model")).unwrap();
        fs::write(
            root.path().join(".agents/skills/vxl-model/SKILL.md"),
            "hand written",
        )
        .unwrap();

        install(&root, SkillTarget::Agents);

        assert_eq!(
            read(&root, ".agents/skills/vxl-model/SKILL.md"),
            AgentSkill::VxlModel.skill_md()
        );
    }

    #[cfg(unix)]
    #[test]
    fn install_to_claude_lands_in_agents_through_a_link() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".agents/skills")).unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        symlink("../.agents/skills", root.path().join(".claude/skills")).unwrap();

        install(&root, SkillTarget::Claude);

        assert_eq!(
            read(&root, ".agents/skills/vxl-model/SKILL.md"),
            AgentSkill::VxlModel.skill_md()
        );
    }
}
