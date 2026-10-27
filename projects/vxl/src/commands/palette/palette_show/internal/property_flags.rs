use crate::commands::PropertyFlag;
use clap::{Arg, ArgAction, ArgMatches, Args, Command, Error as ClapError, FromArgMatches};

/// The `--property` and `--properties-from` occurrences in line order, kept
/// together because each `--properties-from` appends its profile's selectors
/// at the flag's position among the `--property` selectors.
#[derive(Clone, Debug, Default)]
pub struct PropertyFlags {
    pub(crate) entries: Vec<PropertyFlag>,
}

impl PropertyFlags {
    /// Whether any occurrence reads a profile.
    pub(crate) fn uses_profiles(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry, PropertyFlag::PropertiesFrom(_)))
    }
}

impl Args for PropertyFlags {
    fn augment_args(command: Command) -> Command {
        command
            .arg(
                Arg::new("properties_from")
                    .value_name("profile")
                    .long("properties-from")
                    .action(ArgAction::Append)
                    .help(
                        "Appends a profile's selectors at the flag's position, with its \
                         `propertiesFrom` imports first. The profile's layout stays behind. \
                         Repeatable",
                    ),
            )
            .arg(
                Arg::new("property")
                    .value_names(["palette", "property", "presentation", "reading"])
                    .long("property")
                    .num_args(4)
                    .action(ArgAction::Append)
                    .help(
                        "A selector naming a value collection, four fields: `<palette> \
                         <property> <presentation> <reading>`. The palette is an integer, an \
                         inclusive `a-b` range, or `*`; the property a key with an optional \
                         `.r`/`.g`/`.b`/`.a` or `.x`/`.y`/`.z`/`.w` component, or `*`; the \
                         presentation `auto`, `swatch`, `swatch-value`, or `value`; the reading \
                         `auto`, `linear-float`, `plain`, `srgb-float`, or `srgb-hex`. Defaults \
                         to `'*' '*' auto auto` when neither a flag nor a profile selects \
                         anything. Repeatable",
                    ),
            )
    }

    fn augment_args_for_update(command: Command) -> Command {
        Self::augment_args(command)
    }
}

impl FromArgMatches for PropertyFlags {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, ClapError> {
        let mut indexed: Vec<(usize, PropertyFlag)> = Vec::new();

        if let (Some(indices), Some(names)) = (
            matches.indices_of("properties_from"),
            matches.get_many::<String>("properties_from"),
        ) {
            indexed.extend(
                indices
                    .zip(names.cloned())
                    .map(|(index, name)| (index, PropertyFlag::PropertiesFrom(name))),
            );
        }

        if let (Some(indices), Some(fields)) = (
            matches.indices_of("property"),
            matches.get_many::<String>("property"),
        ) {
            let indices: Vec<_> = indices.collect();
            let fields: Vec<_> = fields.cloned().collect();

            // clap fixes each occurrence at four values, so the flattened lists
            // chunk cleanly by occurrence.
            for (indices, fields) in indices.chunks(4).zip(fields.chunks(4)) {
                let fields = fields
                    .to_vec()
                    .try_into()
                    .expect("clap fixes each occurrence at four values");

                indexed.push((indices[0], PropertyFlag::Property(fields)));
            }
        }

        indexed.sort_by_key(|(index, _)| *index);

        Ok(PropertyFlags {
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
    use crate::commands::{PropertyFlag, PropertyFlags};
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        property_flags: PropertyFlags,
    }

    fn fields(fields: [&str; 4]) -> [String; 4] {
        fields.map(str::to_owned)
    }

    #[test]
    fn the_occurrences_keep_their_line_order() {
        let cli = Cli::try_parse_from([
            "cli",
            "--property",
            "0",
            "tint",
            "value",
            "plain",
            "--properties-from",
            "pbr",
            "--property",
            "*",
            "ior",
            "value",
            "plain",
        ])
        .unwrap();

        assert_eq!(
            cli.property_flags.entries,
            [
                PropertyFlag::Property(fields(["0", "tint", "value", "plain"])),
                PropertyFlag::PropertiesFrom("pbr".to_owned()),
                PropertyFlag::Property(fields(["*", "ior", "value", "plain"])),
            ]
        );
        assert!(cli.property_flags.uses_profiles());
    }

    #[test]
    fn selectors_alone_read_no_profile() {
        let cli = Cli::try_parse_from(["cli", "--property", "*", "*", "auto", "auto"]).unwrap();

        assert!(!cli.property_flags.uses_profiles());
        assert!(
            Cli::try_parse_from(["cli"])
                .unwrap()
                .property_flags
                .entries
                .is_empty()
        );
    }

    #[test]
    fn a_short_occurrence_errors() {
        assert!(Cli::try_parse_from(["cli", "--property", "0", "tint", "value"]).is_err());
    }
}
