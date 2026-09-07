use crate::CliValue;
use clap::builder::{MapValueParser, PossibleValue, PossibleValuesParser, TypedValueParser};

/// The clap value parser for a [`CliValue`]: `--help` lists each variant's name
/// with its help line, and an unlisted name is a usage error.
pub fn cli_value_parser<T: CliValue>() -> MapValueParser<PossibleValuesParser, fn(String) -> T> {
    let values = T::VARIANTS
        .iter()
        .map(|variant| PossibleValue::new(variant.name()).help(variant.help()));

    let variant: fn(String) -> T = accepted_variant;

    PossibleValuesParser::new(values).map(variant)
}

/// The variant behind a name the possible-values parser accepted.
fn accepted_variant<T: CliValue>(name: String) -> T {
    T::parse(&name).expect("PossibleValuesParser accepted a listed name")
}
