use crate::{
    CreateDirAll, CreateDirLink, Dependencies, ReadPathKind, Result, WriteStdout,
    commands::{CLAUDE_DIR, ClaudeLink, IntegrationRoot, link_claude, read_claude_link},
};
use clap::Parser;
use std::{io::Error as IOError, path::Path};

const NEW_HELP: &str = "\
Claude Code reads `.claude`. The link points Claude Code at `.agents`, the
directory coding agents share. Codex reads `.agents/skills` itself and needs no
link.

On a real `.claude` directory the link fails and the directory stays.
`integration agents-link move` moves its contents into `.agents` first.";

/// Links a missing `.claude` to `.agents`.
#[derive(Clone, Debug, Parser)]
#[command(name = "new", after_help = NEW_HELP)]
pub struct IntegrationAgentsLinkNew {
    #[command(flatten)]
    root: IntegrationRoot,
}

impl IntegrationAgentsLinkNew {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let root = self.root.resolve(&dependencies)?;

        link_new(&dependencies, &root)
    }
}

fn link_new(
    dependencies: &(impl CreateDirAll + CreateDirLink + ReadPathKind + WriteStdout),
    root: &Path,
) -> Result<()> {
    let claude = root.join(CLAUDE_DIR);

    let problem = match read_claude_link(dependencies, root)? {
        ClaudeLink::Missing => None,

        ClaudeLink::Linked => {
            let line = format!("{} already links to .agents\n", claude.display());

            return Ok(dependencies.write_stdout(line.as_bytes())?);
        }

        ClaudeLink::LinkedElsewhere { target } => Some(format!(
            "{} links to {}, not .agents",
            claude.display(),
            target.display()
        )),

        ClaudeLink::Directory => Some(format!(
            "{} is a directory; `vxl integration agents-link move` moves its contents into \
             .agents and links it",
            claude.display()
        )),

        ClaudeLink::File => Some(format!("{} is a file", claude.display())),
    };

    if let Some(problem) = problem {
        return Err(IOError::other(problem).into());
    }

    let link = link_claude(dependencies, root)?;

    let line = format!("linked {} to .agents\n", link.display());

    Ok(dependencies.write_stdout(line.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use crate::{
        CapturedStdout,
        commands::integration::integration_agents_link::integration_agents_link_new::link_new,
    };
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn new(root: &TempDir) -> Result<String, String> {
        let dependencies = CapturedStdout::default();

        link_new(&dependencies, root.path()).map_err(|e| e.to_string())?;

        Ok(dependencies.stdout())
    }

    #[cfg(unix)]
    #[test]
    fn new_creates_agents_and_a_relative_link_to_it() {
        let root = TempDir::new().unwrap();

        let stdout = new(&root).unwrap();

        let link = root.path().join(".claude");
        assert_eq!(fs::read_link(&link).unwrap(), Path::new(".agents"));
        assert!(root.path().join(".agents").is_dir());
        assert_eq!(stdout, format!("linked {} to .agents\n", link.display()));
    }

    #[cfg(unix)]
    #[test]
    fn new_leaves_an_existing_link_and_says_so() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join(".agents")).unwrap();
        symlink(".agents", root.path().join(".claude")).unwrap();

        let stdout = new(&root).unwrap();

        assert!(stdout.ends_with(".claude already links to .agents\n"));
    }

    #[test]
    fn new_fails_on_a_directory_and_points_to_move() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude/skills")).unwrap();

        let error = new(&root).unwrap_err();

        assert!(error.ends_with(
            ".claude is a directory; `vxl integration agents-link move` moves its contents \
             into .agents and links it"
        ));
        assert!(root.path().join(".claude/skills").is_dir());
        assert!(!root.path().join(".agents").exists());
    }

    #[cfg(unix)]
    #[test]
    fn new_fails_on_a_link_elsewhere() {
        let root = TempDir::new().unwrap();
        symlink("elsewhere", root.path().join(".claude")).unwrap();

        let error = new(&root).unwrap_err();

        assert!(error.ends_with(".claude links to elsewhere, not .agents"));
    }

    #[test]
    fn new_fails_on_a_file() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join(".claude"), "").unwrap();

        let error = new(&root).unwrap_err();

        assert!(error.ends_with(".claude is a file"));
    }
}
