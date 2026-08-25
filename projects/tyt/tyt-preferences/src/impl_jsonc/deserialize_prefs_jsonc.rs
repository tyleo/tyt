use crate::parse_options;
use jsonc_parser::parse_to_serde_value;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    io::{Error as IOError, ErrorKind, Result as IOResult},
    str,
};

/// Deserializes the `key` section from a config file's JSONC bytes. The
/// dialect is JSON plus comments and trailing commas. Returns `None` if the
/// file is empty or the section is absent. A ready-made body for
/// `DeserializePrefs` impls.
pub fn deserialize_prefs_jsonc<T: DeserializeOwned>(
    config_jsonc: &[u8],
    key: &str,
) -> IOResult<Option<T>> {
    let text = str::from_utf8(config_jsonc).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    let value: Option<Value> = parse_to_serde_value(text, &parse_options())
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    let Some(section) = value.as_ref().and_then(|value| value.get(key)) else {
        return Ok(None);
    };

    serde_json::from_value(section.clone())
        .map(Some)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use crate::deserialize_prefs_jsonc;
    use serde_json::{Value, json};

    #[test]
    fn reads_a_section_with_comments_and_trailing_commas() {
        let config = br#"{
  // comment
  "fs": {
    "flag": true,
  },
}
"#;

        let section: Option<Value> = deserialize_prefs_jsonc(config, "fs").unwrap();

        assert_eq!(section, Some(json!({ "flag": true })));
    }

    #[test]
    fn returns_none_for_an_absent_section() {
        let section: Option<Value> = deserialize_prefs_jsonc(b"{ \"fs\": {} }", "oai").unwrap();

        assert_eq!(section, None);
    }

    #[test]
    fn returns_none_for_a_comments_only_file() {
        let section: Option<Value> = deserialize_prefs_jsonc(b"// comment\n", "fs").unwrap();

        assert_eq!(section, None);
    }

    #[test]
    fn rejects_single_quoted_strings() {
        let result: super::IOResult<Option<Value>> = deserialize_prefs_jsonc(b"{ 'fs': {} }", "fs");

        assert!(result.is_err());
    }
}
