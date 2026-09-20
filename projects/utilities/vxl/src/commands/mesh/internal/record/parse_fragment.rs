use crate::{Error, Result};
use vox_value_language::parse;

/// The program fragment `text`, which `origin` holds, with its terminator
/// appended. An all-whitespace fragment errors, as does one that does not
/// parse.
pub(crate) fn parse_fragment(origin: &str, text: &str) -> Result<String> {
    if text.trim().is_empty() {
        return Err(Error::usage(format!(
            "{origin} holds only whitespace where bindings go"
        )));
    }

    let fragment = format!("{text};");

    parse(&fragment).map_err(|error| {
        Error::usage(format!(
            "{origin} holds `{text}`, which does not parse: {error}"
        ))
    })?;

    Ok(fragment)
}

#[cfg(test)]
mod tests {
    use super::parse_fragment;

    #[test]
    fn a_fragment_gains_its_terminator() {
        assert_eq!(parse_fragment("--value", "a = 1").unwrap(), "a = 1;");
        assert_eq!(
            parse_fragment("--value", "a = 1; b = a;").unwrap(),
            "a = 1; b = a;;"
        );
    }

    #[test]
    fn whitespace_and_broken_fragments_error_at_their_origin() {
        let error = parse_fragment("--value", "  ").unwrap_err().to_string();
        assert!(error.contains("--value"), "{error}");

        let error = parse_fragment("the profile `x`'s values entry 0", "a =")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `x`'s values entry 0"),
            "{error}"
        );
        assert!(error.contains("`a =`"), "{error}");
    }
}
