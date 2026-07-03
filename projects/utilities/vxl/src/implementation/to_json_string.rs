use crate::Result;
use serde_json::Value;
use std::io::Error as IOError;

/// Serializes `value` as a JSON string ending in a newline, pretty or compact,
/// the form the read reports share so their JSON reads alike.
pub(crate) fn to_json_string(value: &Value, pretty: bool) -> Result<String> {
    let mut output = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .map_err(IOError::other)?;
    output.push('\n');
    Ok(output)
}
