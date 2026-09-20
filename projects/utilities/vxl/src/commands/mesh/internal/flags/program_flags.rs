use crate::commands::ProgramFlag;
use clap::{Arg, ArgAction, ArgMatches, Args, Command, Error as ClapError, FromArgMatches};

/// The `--value` and `--values-from` occurrences in line order, kept
/// together because each `--values-from` appends its profile's bindings at
/// the flag's position among the `--value` fragments.
#[derive(Clone, Debug, Default)]
pub(crate) struct ProgramFlags {
    pub(crate) entries: Vec<ProgramFlag>,
}

impl ProgramFlags {
    /// Whether any occurrence reads a profile.
    pub(crate) fn uses_profiles(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry, ProgramFlag::ValuesFrom(_)))
    }
}

impl Args for ProgramFlags {
    fn augment_args(command: Command) -> Command {
        command
            .arg(
                Arg::new("value")
                    .value_name("bindings")
                    .long("value")
                    .action(ArgAction::Append)
                    .help(
                        "One or more statements of the value language defining values the \
                         writers and slots can reference. Every property of the effective \
                         palette enters the program as a name. Every occurrence joins the \
                         program in order. Repeatable",
                    ),
            )
            .arg(
                Arg::new("values_from")
                    .value_name("profile")
                    .long("values-from")
                    .action(ArgAction::Append)
                    .help(
                        "Appends a profile's bindings to the program at the flag's position, \
                         the profile's `valuesFrom` imports first. Any writer elements the \
                         profile holds stay behind. Repeatable",
                    ),
            )
    }

    fn augment_args_for_update(command: Command) -> Command {
        Self::augment_args(command)
    }
}

impl FromArgMatches for ProgramFlags {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, ClapError> {
        let mut indexed: Vec<(usize, ProgramFlag)> = Vec::new();

        let mut collect = |id: &str, flag: fn(String) -> ProgramFlag| {
            if let (Some(indices), Some(values)) =
                (matches.indices_of(id), matches.get_many::<String>(id))
            {
                indexed.extend(
                    indices
                        .zip(values.cloned())
                        .map(|(index, value)| (index, flag(value))),
                );
            }
        };

        collect("value", ProgramFlag::Value);
        collect("values_from", ProgramFlag::ValuesFrom);

        indexed.sort_by_key(|(index, _)| *index);

        Ok(ProgramFlags {
            entries: indexed.into_iter().map(|(_, flag)| flag).collect(),
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), ClapError> {
        *self = Self::from_arg_matches(matches)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ProgramFlags;
    use crate::commands::ProgramFlag;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        program_flags: ProgramFlags,
    }

    #[test]
    fn the_occurrences_keep_their_line_order() {
        let cli = Cli::try_parse_from([
            "cli",
            "--value",
            "a = 1",
            "--values-from",
            "orm",
            "--value",
            "b = 2",
            "--values-from",
            "emissive",
        ])
        .unwrap();

        assert_eq!(
            cli.program_flags.entries,
            [
                ProgramFlag::Value("a = 1".to_owned()),
                ProgramFlag::ValuesFrom("orm".to_owned()),
                ProgramFlag::Value("b = 2".to_owned()),
                ProgramFlag::ValuesFrom("emissive".to_owned()),
            ]
        );
        assert!(cli.program_flags.uses_profiles());
    }

    #[test]
    fn values_alone_read_no_profile() {
        let cli = Cli::try_parse_from(["cli", "--value", "a = 1"]).unwrap();

        assert!(!cli.program_flags.uses_profiles());
        assert!(
            Cli::try_parse_from(["cli"])
                .unwrap()
                .program_flags
                .entries
                .is_empty()
        );
    }
}
