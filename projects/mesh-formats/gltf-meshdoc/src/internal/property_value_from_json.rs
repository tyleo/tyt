use crate::{Error, Result};
use branded_id::U32Id;
use meshdoc::{BMeshFile, BMeshTexture, MeshPropertyValue, MeshTextureRef};
use serde_json::Value;

/// The property value a `vxl.values` entry named `name` holds, the inverse
/// of [`property_value_to_json`](crate::property_value_to_json). A texture
/// entry holds a texture index into `texture_ids`, and a file entry a
/// relative URI that `file_id_for_uri` resolves. Errors on a shape no
/// property value has.
pub fn property_value_from_json(
    name: &str,
    value: &Value,
    texture_ids: &[U32Id<BMeshTexture>],
    file_id_for_uri: &impl Fn(&str) -> Result<U32Id<BMeshFile>>,
) -> Result<MeshPropertyValue> {
    let malformed = |what: &str| {
        Error::invalid(format!(
            "extras `vxl.values.{name}` {what}, which no property value holds"
        ))
    };

    Ok(match value {
        Value::Bool(value) => MeshPropertyValue::Bool(*value),
        Value::Number(number) => match number.as_i64() {
            Some(value) => MeshPropertyValue::Int(value),
            None => MeshPropertyValue::Float(
                number
                    .as_f64()
                    .ok_or_else(|| malformed("is a number outside f64"))?,
            ),
        },
        Value::String(value) => MeshPropertyValue::Text(value.clone()),
        Value::Array(entries) => list_from_json(entries).ok_or_else(|| malformed("mixes kinds"))?,
        Value::Object(object) => {
            if let Some(uri) = object.get("uri") {
                let uri = uri
                    .as_str()
                    .ok_or_else(|| malformed("has a `uri` that is not a string"))?;
                MeshPropertyValue::File(file_id_for_uri(uri)?)
            } else if let Some(index) = object.get("index") {
                let index = index
                    .as_u64()
                    .and_then(|index| usize::try_from(index).ok())
                    .ok_or_else(|| malformed("has an `index` that is not a texture index"))?;
                let texture_id = *texture_ids
                    .get(index)
                    .ok_or_else(|| malformed("names a texture index past the textures"))?;
                let uv_stream = match object.get("texCoord") {
                    None => 0,
                    Some(set) => set
                        .as_u64()
                        .and_then(|set| u32::try_from(set).ok())
                        .ok_or_else(|| malformed("has a `texCoord` that is not a set"))?,
                };
                MeshPropertyValue::Texture(MeshTextureRef {
                    texture_id,
                    uv_stream_id: U32Id::from_u32(uv_stream),
                })
            } else {
                return Err(malformed("is an object with neither `uri` nor `index`"));
            }
        }
        Value::Null => return Err(malformed("is null")),
    })
}

/// The list value of `entries`, all of one kind, or `None` when they mix.
/// An empty list, or a list of empty rows, reads as float.
fn list_from_json(entries: &[Value]) -> Option<MeshPropertyValue> {
    if entries.is_empty() {
        return Some(MeshPropertyValue::Floats(vec![]));
    }

    let rows: Option<Vec<&[Value]>> = entries
        .iter()
        .map(|entry| entry.as_array().map(Vec::as_slice))
        .collect();

    let Some(rows) = rows else {
        if let Some(values) = leaves(entries, Value::as_bool) {
            return Some(MeshPropertyValue::Bools(values));
        }

        if let Some(values) = leaves(entries, text) {
            return Some(MeshPropertyValue::Texts(values));
        }

        if let Some(values) = leaves(entries, Value::as_i64) {
            return Some(MeshPropertyValue::Ints(values));
        }

        return leaves(entries, Value::as_f64).map(MeshPropertyValue::Floats);
    };

    if rows.iter().all(|row| row.is_empty()) {
        return Some(MeshPropertyValue::FloatRows(vec![vec![]; rows.len()]));
    }

    if let Some(rows) = rows_of(&rows, Value::as_bool) {
        return Some(MeshPropertyValue::BoolRows(rows));
    }

    if let Some(rows) = rows_of(&rows, text) {
        return Some(MeshPropertyValue::TextRows(rows));
    }

    if let Some(rows) = rows_of(&rows, Value::as_i64) {
        return Some(MeshPropertyValue::IntRows(rows));
    }

    rows_of(&rows, Value::as_f64).map(MeshPropertyValue::FloatRows)
}

/// Every entry of `values` read by `leaf`, or `None` when one is another
/// kind.
fn leaves<T>(values: &[Value], leaf: impl Fn(&Value) -> Option<T>) -> Option<Vec<T>> {
    values.iter().map(leaf).collect()
}

/// Every row of `rows` read by `leaf`, or `None` when a leaf is another kind.
fn rows_of<T>(rows: &[&[Value]], leaf: impl Fn(&Value) -> Option<T>) -> Option<Vec<Vec<T>>> {
    rows.iter().map(|row| leaves(row, &leaf)).collect()
}

/// A string leaf, owned.
fn text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}
