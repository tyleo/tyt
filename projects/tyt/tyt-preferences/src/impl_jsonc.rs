use crate::{DeserializePrefs, SerializePrefs};
use jsonc_parser::{
    ParseOptions,
    cst::{CstInputValue, CstRootNode},
    parse_to_serde_value,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    io::{Error as IOError, ErrorKind, Result as IOResult},
    str,
};

/// JSONC codec: JSON plus comments and trailing commas. Reading treats an empty
/// or comments-only file as having no sections. Writing preserves the comments
/// and formatting of every untouched section.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsoncCodec;

impl<T: DeserializeOwned> DeserializePrefs<T> for JsoncCodec {
    fn deserialize_prefs(&self, config: &[u8], key: &str) -> IOResult<Option<T>> {
        let text = str::from_utf8(config).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        let value: Option<Value> = parse_to_serde_value(text, &parse_options())
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        let Some(section) = value.as_ref().and_then(|value| value.get(key)) else {
            return Ok(None);
        };

        serde_json::from_value(section.clone())
            .map(Some)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
    }
}

impl<T: Serialize> SerializePrefs<T> for JsoncCodec {
    fn serialize_prefs(&self, value: &T, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>> {
        let text = match existing {
            Some(bytes) => {
                str::from_utf8(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?
            }
            None => "",
        };

        let root = CstRootNode::parse(text, &parse_options())
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        let Some(object) = root.object_value_or_create() else {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                "config file root must be a JSON object",
            ));
        };

        let section = cst_input_value(serde_json::to_value(value).map_err(IOError::other)?);

        match object.get(key) {
            Some(prop) => prop.set_value(section),
            None => {
                object.append(key, section);
            }
        }

        Ok(root.to_string().into_bytes())
    }
}

/// The accepted JSONC dialect: JSON plus comments and trailing commas.
fn parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

fn cst_input_value(value: Value) -> CstInputValue {
    match value {
        Value::Null => CstInputValue::Null,
        Value::Bool(value) => CstInputValue::Bool(value),
        Value::Number(value) => CstInputValue::Number(value.to_string()),
        Value::String(value) => CstInputValue::String(value),
        Value::Array(values) => {
            CstInputValue::Array(values.into_iter().map(cst_input_value).collect())
        }
        Value::Object(values) => CstInputValue::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, cst_input_value(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{DeserializePrefs as _, JsoncCodec, SerializePrefs as _};
    use serde_json::{Value, json};
    use std::io::Result as IOResult;

    #[test]
    fn reads_a_section_with_comments_and_trailing_commas() {
        let config = br#"{
  // comment
  "fs": {
    "flag": true,
  },
}
"#;

        let section: Option<Value> = JsoncCodec.deserialize_prefs(config, "fs").unwrap();

        assert_eq!(section, Some(json!({ "flag": true })));
    }

    #[test]
    fn returns_none_for_an_absent_section() {
        let section: Option<Value> = JsoncCodec
            .deserialize_prefs(b"{ \"fs\": {} }", "oai")
            .unwrap();

        assert_eq!(section, None);
    }

    #[test]
    fn returns_none_for_a_comments_only_file() {
        let section: Option<Value> = JsoncCodec.deserialize_prefs(b"// comment\n", "fs").unwrap();

        assert_eq!(section, None);
    }

    #[test]
    fn rejects_single_quoted_strings() {
        let result: IOResult<Option<Value>> = JsoncCodec.deserialize_prefs(b"{ 'fs': {} }", "fs");

        assert!(result.is_err());
    }

    #[test]
    fn replaces_the_section_and_preserves_comments() {
        let existing = br#"{
  // comment
  "fs": {
    "old": true
  },
  "oai": {}
}
"#;

        let bytes = JsoncCodec
            .serialize_prefs(&json!({ "new": 1 }), "fs", Some(existing))
            .unwrap();

        let expected = r#"{
  // comment
  "fs": {
    "new": 1
  },
  "oai": {}
}
"#;
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }

    #[test]
    fn appends_an_absent_section() {
        let existing = br#"{
  "oai": {}
}
"#;

        let bytes = JsoncCodec
            .serialize_prefs(&json!({ "new": 1 }), "fs", Some(existing))
            .unwrap();

        let expected = r#"{
  "oai": {},
  "fs": {
    "new": 1
  }
}
"#;
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }

    #[test]
    fn creates_the_file_when_existing_is_none() {
        let bytes = JsoncCodec
            .serialize_prefs(&json!({ "new": 1 }), "fs", None)
            .unwrap();

        let expected = r#"{
  "fs": {
    "new": 1
  }
}
"#;
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }

    #[test]
    fn rejects_a_non_object_root() {
        let result = JsoncCodec.serialize_prefs(&json!({ "new": 1 }), "fs", Some(b"[]\n"));

        assert!(result.is_err());
    }
}
