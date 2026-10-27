use crate::ObjectSelection;
use clap::{Error as ClapError, Parser};

/// A command carrying only the selectors.
#[derive(Debug, Parser)]
struct Cli {
    #[command(flatten)]
    selection: ObjectSelection,
}

/// The selectors parsed from `args`.
pub fn try_parse_object_selection(args: &[&str]) -> Result<ObjectSelection, ClapError> {
    let mut argv = vec!["cli"];
    argv.extend_from_slice(args);
    Cli::try_parse_from(argv).map(|cli| cli.selection)
}
