use crate::{Dependencies, completion_script};
use clap::Command;
use clap_complete::Shell;
use std::{
    io::{Error as IOError, Result as IOResult},
    path::{Path, PathBuf},
};

/// Writes `shell`'s completions for the binary `bin` where `shell` loads them,
/// then reports the path and any line the shell's startup file needs.
pub fn install_completion(
    dependencies: &impl Dependencies,
    mut command: Command,
    bin: &str,
    shell: Shell,
) -> IOResult<()> {
    let home = home_dir(dependencies)?;

    let (path, startup) = match shell {
        Shell::Bash => {
            let data_home = xdg_dir(dependencies, "XDG_DATA_HOME", &home, ".local/share")?;

            (
                data_home.join("bash-completion/completions").join(bin),
                None,
            )
        }

        Shell::Elvish => {
            let data_home = xdg_dir(dependencies, "XDG_DATA_HOME", &home, ".local/share")?;

            let path = data_home.join(format!("elvish/completions/{bin}.elv"));

            let line = format!("eval (slurp < {})", path.display());

            (path, Some(("elvish's rc.elv", line)))
        }

        Shell::Fish => {
            let config_home = xdg_dir(dependencies, "XDG_CONFIG_HOME", &home, ".config")?;

            (
                config_home.join(format!("fish/completions/{bin}.fish")),
                None,
            )
        }

        Shell::PowerShell => {
            let data_home = xdg_dir(dependencies, "XDG_DATA_HOME", &home, ".local/share")?;

            let path = data_home.join(format!("powershell/completions/{bin}.ps1"));

            let line = format!(". \"{}\"", path.display());

            (path, Some(("$PROFILE", line)))
        }

        Shell::Zsh => {
            let dir = home.join(".zsh/completions");

            let line = format!("fpath=({} $fpath)", dir.display());

            (
                dir.join(format!("_{bin}")),
                Some(("~/.zshrc before compinit", line)),
            )
        }

        shell => {
            return Err(IOError::other(format!(
                "installing {shell} completions is not supported; \
                 `integration completion` lists the shells it supports"
            )));
        }
    };

    dependencies.write_file(&path, &completion_script(&mut command, bin, shell))?;

    let mut report = format!("installed {}\n", path.display());

    if let Some((file, line)) = startup {
        report.push_str(&format!(
            "add this line to {file} unless it is there:\n  {line}\n"
        ));
    }

    dependencies.write_stdout(report.as_bytes())
}

/// The home directory from `HOME`, or `USERPROFILE` on Windows.
fn home_dir(dependencies: &impl Dependencies) -> IOResult<PathBuf> {
    let Some(home) = dependencies
        .read_env_var("HOME")
        .or_else(|| dependencies.read_env_var("USERPROFILE"))
    else {
        return Err(IOError::other("neither HOME nor USERPROFILE is set"));
    };

    Ok(PathBuf::from(home))
}

/// The XDG base directory in `var`, or `home` joined with `default` when `var`
/// is unset or empty, as the XDG spec has it.
fn xdg_dir(
    dependencies: &impl Dependencies,
    var: &str,
    home: &Path,
    default: &str,
) -> IOResult<PathBuf> {
    let Some(value) = dependencies
        .read_env_var(var)
        .filter(|value| !value.is_empty())
    else {
        return Ok(home.join(default));
    };

    let dir = PathBuf::from(value);

    if dir.is_relative() {
        return Err(IOError::other(format!(
            "{var} is the relative path {}, and the XDG spec takes only absolute paths",
            dir.display()
        )));
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use crate::{Dependencies, DependenciesImpl, completion_script, install_completion};
    use clap::{Command, CommandFactory, Parser};
    use clap_complete::Shell;
    use std::{
        cell::RefCell,
        collections::HashMap,
        ffi::OsString,
        fs,
        io::Result as IOResult,
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    /// A test binary.
    #[derive(Parser)]
    #[command(name = "demo")]
    struct Demo {
        /// A flag to complete.
        #[arg(value_name = "flag", long)]
        flag: bool,
    }

    /// Set environment variables, real files, and a captured standard output.
    #[derive(Default)]
    struct StandIn {
        env: HashMap<&'static str, OsString>,

        stdout: RefCell<Vec<u8>>,
    }

    impl Dependencies for StandIn {
        fn read_env_var(&self, name: &str) -> Option<OsString> {
            self.env.get(name).cloned()
        }

        fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
            DependenciesImpl.write_file(path, bytes)
        }

        fn write_stdout(&self, bytes: &[u8]) -> IOResult<()> {
            self.stdout.borrow_mut().extend_from_slice(bytes);

            Ok(())
        }
    }

    fn stand_in(env: &[(&'static str, &Path)]) -> StandIn {
        StandIn {
            env: env
                .iter()
                .map(|(name, path)| (*name, path.as_os_str().to_owned()))
                .collect(),
            ..StandIn::default()
        }
    }

    fn command() -> Command {
        Demo::command()
    }

    fn install(dependencies: &StandIn, shell: Shell) -> Result<String, String> {
        install_completion(dependencies, command(), "demo", shell).map_err(|e| e.to_string())?;

        Ok(String::from_utf8(dependencies.stdout.take()).unwrap())
    }

    fn assert_script(path: &Path, shell: Shell) {
        let script = completion_script(&mut command(), "demo", shell);

        assert_eq!(fs::read(path).unwrap(), script, "{}", path.display());
    }

    #[test]
    fn install_writes_each_shell_under_home_and_reports_any_startup_line() {
        let home = TempDir::new().unwrap();
        let dependencies = stand_in(&[("HOME", home.path())]);
        let at = |path: &str| home.path().join(path);

        let cases: [(Shell, PathBuf, Option<String>); 5] = [
            (
                Shell::Bash,
                at(".local/share/bash-completion/completions/demo"),
                None,
            ),
            (
                Shell::Elvish,
                at(".local/share/elvish/completions/demo.elv"),
                Some(format!(
                    "add this line to elvish's rc.elv unless it is there:\n  eval (slurp < {})\n",
                    at(".local/share/elvish/completions/demo.elv").display()
                )),
            ),
            (Shell::Fish, at(".config/fish/completions/demo.fish"), None),
            (
                Shell::PowerShell,
                at(".local/share/powershell/completions/demo.ps1"),
                Some(format!(
                    "add this line to $PROFILE unless it is there:\n  . \"{}\"\n",
                    at(".local/share/powershell/completions/demo.ps1").display()
                )),
            ),
            (
                Shell::Zsh,
                at(".zsh/completions/_demo"),
                Some(format!(
                    "add this line to ~/.zshrc before compinit unless it is there:\n  \
                     fpath=({} $fpath)\n",
                    at(".zsh/completions").display()
                )),
            ),
        ];

        for (shell, path, startup) in cases {
            let stdout = install(&dependencies, shell).unwrap();

            assert_script(&path, shell);
            let installed = format!("installed {}\n", path.display());
            assert_eq!(stdout, installed + startup.as_deref().unwrap_or(""));
        }
    }

    #[test]
    fn install_follows_the_xdg_base_directories() {
        let home = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let dependencies = stand_in(&[
            ("HOME", home.path()),
            ("XDG_DATA_HOME", data.path()),
            ("XDG_CONFIG_HOME", config.path()),
        ]);

        install(&dependencies, Shell::Bash).unwrap();
        install(&dependencies, Shell::Fish).unwrap();

        assert_script(
            &data.path().join("bash-completion/completions/demo"),
            Shell::Bash,
        );
        assert_script(
            &config.path().join("fish/completions/demo.fish"),
            Shell::Fish,
        );
    }

    #[test]
    fn install_treats_an_empty_xdg_variable_as_unset() {
        let home = TempDir::new().unwrap();
        let dependencies = stand_in(&[("HOME", home.path()), ("XDG_DATA_HOME", Path::new(""))]);

        install(&dependencies, Shell::Bash).unwrap();

        assert_script(
            &home
                .path()
                .join(".local/share/bash-completion/completions/demo"),
            Shell::Bash,
        );
    }

    #[test]
    fn install_fails_on_a_relative_xdg_variable() {
        let home = TempDir::new().unwrap();
        let dependencies = stand_in(&[
            ("HOME", home.path()),
            ("XDG_CONFIG_HOME", Path::new("config")),
        ]);

        let error = install(&dependencies, Shell::Fish).unwrap_err();

        assert_eq!(
            error,
            "XDG_CONFIG_HOME is the relative path config, and the XDG spec takes only \
             absolute paths"
        );
    }

    #[test]
    fn install_uses_userprofile_without_home() {
        let home = TempDir::new().unwrap();
        let dependencies = stand_in(&[("USERPROFILE", home.path())]);

        install(&dependencies, Shell::Zsh).unwrap();

        assert_script(&home.path().join(".zsh/completions/_demo"), Shell::Zsh);
    }

    #[test]
    fn install_fails_without_a_home_directory() {
        let error = install(&StandIn::default(), Shell::Zsh).unwrap_err();

        assert_eq!(error, "neither HOME nor USERPROFILE is set");
    }
}
