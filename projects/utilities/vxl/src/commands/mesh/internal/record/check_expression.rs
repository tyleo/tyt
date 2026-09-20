use crate::{Error, Result};
use vox_value_language::parse_expression;

/// Errors unless `text`, which `origin` holds, parses as one expression.
pub(crate) fn check_expression(origin: &str, text: &str) -> Result<()> {
    parse_expression(text).map(drop).map_err(|error| {
        Error::usage(format!(
            "{origin} holds `{text}`, which does not parse as an expression: {error}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::check_expression;

    #[test]
    fn a_broken_expression_errors_at_its_origin() {
        assert!(check_expression("--primitive", "emissiveStrength > 0").is_ok());

        let error = check_expression("--primitive", "1 +")
            .unwrap_err()
            .to_string();
        assert!(error.contains("--primitive"), "{error}");
        assert!(error.contains("`1 +`"), "{error}");
    }
}
