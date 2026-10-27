use crate::{
    CreateDirAll, CreateDirLink, Dependencies, ListDir, PathKind, ReadPathKind, RemoveDir,
    RenamePath, Result, WriteStdout,
    commands::{
        AGENTS_DIR, CLAUDE_DIR, ClaudeLink, IntegrationRoot, link_claude, read_claude_link,
    },
};
use clap::Parser;
use std::{io::Error as IOError, path::Path};

const MOVE_HELP: &str = "\
Claude Code reads `.claude`. The link points Claude Code at `.agents`, the
directory coding agents share. Codex reads `.agents/skills` itself and needs no
link.

Everything in `.claude` moves into `.agents` before the link replaces the
directory. When a name sits in both directories, the move fails before anything
changes.";

/// Moves everything in a `.claude` directory into `.agents`, then links
/// `.claude` there.
#[derive(Clone, Debug, Parser)]
#[command(name = "move", after_help = MOVE_HELP)]
pub struct IntegrationAgentsLinkMove {
    #[command(flatten)]
    root: IntegrationRoot,
}

impl IntegrationAgentsLinkMove {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let root = self.root.resolve(&dependencies)?;

        link_move(&dependencies, &root)
    }
}

fn link_move(
    dependencies: &(
         impl CreateDirAll
         + CreateDirLink
         + ListDir
         + ReadPathKind
         + RemoveDir
         + RenamePath
         + WriteStdout
     ),
    root: &Path,
) -> Result<()> {
    let claude = root.join(CLAUDE_DIR);

    let problem = match read_claude_link(dependencies, root)? {
        ClaudeLink::Directory => None,

        ClaudeLink::Linked => {
            let line = format!("{} already links to .agents\n", claude.display());

            return Ok(dependencies.write_stdout(line.as_bytes())?);
        }

        ClaudeLink::Missing => Some(format!(
            "{} does not exist; `vxl integration agents-link new` links it",
            claude.display()
        )),

        ClaudeLink::LinkedElsewhere { target } => Some(format!(
            "{} links to {}, not .agents",
            claude.display(),
            target.display()
        )),

        ClaudeLink::File => Some(format!("{} is a file", claude.display())),
    };

    if let Some(problem) = problem {
        return Err(IOError::other(problem).into());
    }

    let agents = root.join(AGENTS_DIR);

    let entries = dependencies.list_dir(&claude)?;

    let mut clashes = Vec::new();

    for entry in &entries {
        let name = entry.path.file_name().expect("a listed entry has a name");

        if dependencies.read_path_kind(&agents.join(name))? != PathKind::Missing {
            clashes.push(name.to_string_lossy().into_owned());
        }
    }

    if !clashes.is_empty() {
        clashes.sort();

        return Err(IOError::other(format!(
            "{} and {} both hold {}",
            claude.display(),
            agents.display(),
            clashes.join(", ")
        ))
        .into());
    }

    dependencies.create_dir_all(&agents)?;

    for entry in &entries {
        let name = entry.path.file_name().expect("a listed entry has a name");

        dependencies.rename_path(&entry.path, &agents.join(name))?;
    }

    dependencies.remove_dir(&claude)?;

    let link = link_claude(dependencies, root)?;

    let noun = if entries.len() == 1 {
        "entry"
    } else {
        "entries"
    };

    let line = format!(
        "moved {} {noun} into {}, then linked {} to .agents\n",
        entries.len(),
        agents.display(),
        link.display()
    );

    Ok(dependencies.write_stdout(line.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use crate::{
        CapturedStdout,
        commands::integration::integration_agents_link::integration_agents_link_move::link_move,
    };
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn link(root: &TempDir) -> Result<String, String> {
        let dependencies = CapturedStdout::default();

        link_move(&dependencies, root.path()).map_err(|e| e.to_string())?;

        Ok(dependencies.stdout())
    }

    fn write(root: &TempDir, path: &str, contents: &str) {
        let path = root.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn read(root: &TempDir, path: &str) -> String {
        fs::read_to_string(root.path().join(path)).unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn move_moves_everything_into_agents_then_links() {
        let root = TempDir::new().unwrap();
        write(&root, ".claude/settings.json", "{}");
        write(&root, ".claude/skills/a/SKILL.md", "a");
        write(&root, ".agents/README.md", "shared");

        let stdout = link(&root).unwrap();

        let claude = root.path().join(".claude");
        assert_eq!(fs::read_link(&claude).unwrap(), Path::new(".agents"));
        assert_eq!(read(&root, ".agents/settings.json"), "{}");
        assert_eq!(read(&root, ".claude/skills/a/SKILL.md"), "a");
        assert_eq!(read(&root, ".claude/README.md"), "shared");
        assert_eq!(
            stdout,
            format!(
                "moved 2 entries into {}, then linked {} to .agents\n",
                root.path().join(".agents").display(),
                claude.display()
            )
        );
    }

    #[test]
    fn move_fails_on_names_in_both_directories_before_moving_anything() {
        let root = TempDir::new().unwrap();
        write(&root, ".claude/settings.json", "claude");
        write(&root, ".claude/skills/a/SKILL.md", "claude a");
        write(&root, ".claude/plugins/p", "claude p");
        write(&root, ".agents/skills/b/SKILL.md", "agents b");
        write(&root, ".agents/plugins/q", "agents q");

        let error = link(&root).unwrap_err();

        assert!(error.ends_with(".agents both hold plugins, skills"));
        assert_eq!(read(&root, ".claude/settings.json"), "claude");
        assert!(!root.path().join(".agents/settings.json").exists());
    }

    #[test]
    fn move_fails_on_a_missing_directory_and_points_to_new() {
        let root = TempDir::new().unwrap();

        let error = link(&root).unwrap_err();

        assert!(
            error.ends_with(".claude does not exist; `vxl integration agents-link new` links it")
        );
        assert!(!root.path().join(".agents").exists());
    }

    #[cfg(unix)]
    #[test]
    fn move_leaves_an_existing_link_and_says_so() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join(".agents")).unwrap();
        symlink(".agents", root.path().join(".claude")).unwrap();

        let stdout = link(&root).unwrap();

        assert!(stdout.ends_with(".claude already links to .agents\n"));
    }
}
