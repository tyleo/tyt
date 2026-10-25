use clap::{
    Arg, Command, Error as ClapError,
    builder::{PossibleValue, StringValueParser, TypedValueParser},
};
use std::ffi::OsStr;

/// The parser for `--font`'s values, which keeps each value as text. Shell
/// completion offers the font kinds for both values because clap gives a flag
/// one list.
#[derive(Clone, Debug)]
pub struct FontFlagParser;

impl TypedValueParser for FontFlagParser {
    type Value = String;

    fn parse_ref(
        &self,
        command: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<String, ClapError> {
        StringValueParser::new().parse_ref(command, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            ["file", "installed"].into_iter().map(PossibleValue::new),
        ))
    }
}
