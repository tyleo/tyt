use crate::Scope;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    IO(IOError),
    NoUserHome,
    NoGitRoot,
    ProfileAlreadyExists { name: String, scope: Scope },
    ProfileNotFound { name: String },
    NoActiveProfile,
    ClaudeNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::NoUserHome => f.write_str("could not determine the user home directory"),
            Error::NoGitRoot => f.write_str(
                "not inside a git repository; --scope=repo and --scope=repo-user require one",
            ),
            Error::ProfileAlreadyExists { name, scope } => {
                write!(f, "profile '{name}' already exists in --scope={scope}")
            }
            Error::ProfileNotFound { name } => write!(f, "profile '{name}' is not defined"),
            Error::NoActiveProfile => f.write_str(
                "no active claude profile; run 'tyt claude set-profile <name>' or pass --profile",
            ),
            Error::ClaudeNotFound => f.write_str(
                "could not find 'claude' on PATH; install Claude Code or adjust your PATH",
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::NoUserHome
            | Error::NoGitRoot
            | Error::ProfileAlreadyExists { .. }
            | Error::ProfileNotFound { .. }
            | Error::NoActiveProfile
            | Error::ClaudeNotFound => None,
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
