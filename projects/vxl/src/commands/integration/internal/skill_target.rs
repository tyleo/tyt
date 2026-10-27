use crate::commands::{AGENTS_DIR, CLAUDE_DIR, SKILLS_DIR};
use clap::ValueEnum;
use std::path::{Path, PathBuf};

/// The skills directory a skill installs into.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SkillTarget {
    /// `.agents/skills`, which Codex reads.
    #[value(name = "agents")]
    Agents,

    /// `.claude/skills`, which Claude Code reads.
    #[value(name = "claude")]
    Claude,
}

impl SkillTarget {
    /// The target's skills directory under `root`.
    pub fn skills_dir(self, root: &Path) -> PathBuf {
        let agent_dir = match self {
            SkillTarget::Agents => AGENTS_DIR,
            SkillTarget::Claude => CLAUDE_DIR,
        };

        root.join(agent_dir).join(SKILLS_DIR)
    }
}
