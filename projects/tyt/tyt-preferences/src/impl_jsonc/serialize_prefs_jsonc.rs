use crate::parse_options;
use jsonc_parser::cst::{CstInputValue, CstRootNode};
use serde::Serialize;
use serde_json::Value;
use std::{
    io::{Error as IOError, ErrorKind, Result as IOResult},
    str,
};

/// Builds the file bytes in which `key` maps to an encoding of `value`, and
/// every other top-level section of `existing` is preserved along with its
/// comments and formatting. Starts from an empty object when `existing` is
/// `None`. A ready-made body for `SerializePrefs` impls.
pub fn serialize_prefs_jsonc<T: Serialize>(
    value: &T,
    key: &str,
    existing: Option<&[u8]>,
) -> IOResult<Vec<u8>> {
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
    use crate::serialize_prefs_jsonc;
    use serde_json::json;

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

        let bytes = serialize_prefs_jsonc(&json!({ "new": 1 }), "fs", Some(existing)).unwrap();

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

        let bytes = serialize_prefs_jsonc(&json!({ "new": 1 }), "fs", Some(existing)).unwrap();

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
        let bytes = serialize_prefs_jsonc(&json!({ "new": 1 }), "fs", None).unwrap();

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
        let result = serialize_prefs_jsonc(&json!({ "new": 1 }), "fs", Some(b"[]\n"));

        assert!(result.is_err());
    }
}
