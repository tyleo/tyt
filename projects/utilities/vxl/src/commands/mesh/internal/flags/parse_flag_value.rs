use crate::{CliValue, Error, Result};

/// Parses a named token of `flag`, listing the accepted names on failure.
pub(crate) fn parse_flag_value<T: CliValue>(flag: &str, text: &str) -> Result<T> {
    T::parse(text).map_err(|reason| Error::usage(format!("{flag}: {reason}")))
}
