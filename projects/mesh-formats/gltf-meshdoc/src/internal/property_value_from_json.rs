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
/// An empty list is an empty float list.
fn list_from_json(entries: &[Value]) -> Option<MeshPropertyValue> {
    if let Some(values) = entries
        .iter()
        .map(Value::as_bool)
        .collect::<Option<Vec<bool>>>()
        && !entries.is_empty()
    {
        return Some(MeshPropertyValue::Bools(values));
    }

    if let Some(values) = entries
        .iter()
        .map(|entry| entry.as_str().map(str::to_owned))
        .collect::<Option<Vec<String>>>()
        && !entries.is_empty()
    {
        return Some(MeshPropertyValue::Texts(values));
    }

    if entries.iter().all(Value::is_number) {
        return Some(
            match entries
                .iter()
                .map(Value::as_i64)
                .collect::<Option<Vec<i64>>>()
            {
                Some(values) if !entries.is_empty() => MeshPropertyValue::Ints(values),
                _ => MeshPropertyValue::Floats(entries.iter().filter_map(Value::as_f64).collect()),
            },
        );
    }

    entries
        .iter()
        .map(|entry| {
            entry
                .as_array()
                .and_then(|row| row.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>())
        })
        .collect::<Option<Vec<Vec<f64>>>>()
        .map(MeshPropertyValue::FloatRows)
}
